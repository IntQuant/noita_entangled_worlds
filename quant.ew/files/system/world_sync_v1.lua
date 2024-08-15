local world_ffi = require("noitapatcher.nsew.world_ffi")
local world = require("noitapatcher.nsew.world")
local rect = require("noitapatcher.nsew.rect")
local ffi = require("ffi")

local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")

local rpc = net.new_rpc_namespace()

local rect_optimiser = rect.Optimiser_new()
local encoded_area = world.EncodedArea()

local world_sync = {}
local pending_send_wd = {}

local bandwidth_per_frame = 1024*1024/8/60 -- 1 MBit per second
local bandwidth_bucket_max = 29000
local bandwidth_bucket = 0

local function send_pending()
    local will_send = {}
    local total_len = 0
    while #pending_send_wd > 0 do
        local packet = table.remove(pending_send_wd, 1)
        local len = string.len(packet)
        if len > bandwidth_bucket then
            table.insert(pending_send_wd, 1, packet)
            break
        end
        bandwidth_bucket = bandwidth_bucket - len
        total_len = total_len + len
        table.insert(will_send, packet)
    end
    if #will_send > 0 then
        -- GamePrint(#pending_send_wd.." "..#will_send.." "..total_len)
        rpc.send_world_data(will_send)
    end
end

function world_sync.on_world_update_host()
    bandwidth_bucket = bandwidth_bucket + bandwidth_per_frame
    if bandwidth_bucket > bandwidth_bucket_max then
        bandwidth_bucket = bandwidth_bucket_max
    end

    local grid_world = world_ffi.get_grid_world()
    local chunk_map = grid_world.vtable.get_chunk_map(grid_world)
    local thread_impl = grid_world.mThreadImpl

    local begin = thread_impl.updated_grid_worlds.begin
    --local end_ = begin + thread_impl.chunk_update_count

    local count = thread_impl.chunk_update_count
    -- GamePrint("w update "..count)
    for i = 0, count - 1 do
        local it = begin[i]

        local start_x = it.update_region.top_left.x
        local start_y = it.update_region.top_left.y
        local end_x = it.update_region.bottom_right.x
        local end_y = it.update_region.bottom_right.y

        start_x = start_x - 1
        start_y = start_y - 1
        end_x = end_x + 1
        end_y = end_y + 2

        -- if i < 9 then
        --     GamePrint(start_x.." "..start_y.." "..end_x.." "..end_y)
        -- end
        local rectangle = rect.Rectangle(start_x, start_y, end_x, end_y)
        rect_optimiser:submit(rectangle)
    end
    for i = 0, tonumber(thread_impl.world_update_params_count) - 1 do
        local wup = thread_impl.world_update_params.begin[i]
        local start_x = wup.update_region.top_left.x
        local start_y = wup.update_region.top_left.y
        local end_x = wup.update_region.bottom_right.x
        local end_y = wup.update_region.bottom_right.y

        local rectangle = rect.Rectangle(start_x, start_y, end_x, end_y)
        rect_optimiser:submit(rectangle)
    end
    if #pending_send_wd == 0 then
        rect_optimiser:scan()

        for crect in rect.parts(rect_optimiser:iterate(), 256) do
            local area
            -- Make sure we don't send chunks that aren't loaded yet, like holy mountains before host got there.
            if DoesWorldExistAt(crect.left, crect.top, crect.right, crect.bottom) then
                area = world.encode_area(chunk_map, crect.left, crect.top, crect.right, crect.bottom, encoded_area)
            else
                -- Will probably need to try again later?
            end
            if area ~= nil then
                local str = ffi.string(area, world.encoded_size(area))
                if string.len(str) > bandwidth_bucket_max then
                    GamePrint("Discarding chunk update, as it is too large to be sent")
                else
                    table.insert(pending_send_wd, str)
                end
            end
        end
        rect_optimiser:reset()
    end
    send_pending()
end

local PixelRun_const_ptr = ffi.typeof("struct PixelRun const*")

function world_sync.handle_world_data(world_data)
    local grid_world = world_ffi.get_grid_world()
    for _, datum in ipairs(world_data) do
        -- GamePrint("Decoding world data "..i)
        local header = ffi.cast("struct EncodedAreaHeader const*", ffi.cast('char const*', datum))
        local runs = ffi.cast(PixelRun_const_ptr, ffi.cast("const char*", datum) + ffi.sizeof(world.EncodedAreaHeader))
        world.decode(grid_world, header, runs)
    end
end

function rpc.send_world_data(world_data)
    if ctx.rpc_peer_id == ctx.host_id then
        world_sync.handle_world_data(world_data)
    end
end

return world_sync