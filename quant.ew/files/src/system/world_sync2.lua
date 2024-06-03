local world_ffi = require("noitapatcher.nsew.world_ffi")
local world = require("noitapatcher.nsew.world")
local rect = require("noitapatcher.nsew.rect")
local ffi = require("ffi")

local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")

-- local rpc = net.new_rpc_namespace()

local rect_optimiser = rect.Optimiser_new()
local encoded_area = world.EncodedArea()

local world_sync = {}

local bandwidth_bucket_max = 29000

local KEY_WORLD_FRAME = 0
local KEY_WORLD_END = 1

local initialized_chunks = {}
local CHUNK_SIZE = 256

function world_sync.on_world_update_host()

    local grid_world = world_ffi.get_grid_world()
    local chunk_map = grid_world.vtable.get_chunk_map(grid_world)
    local thread_impl = grid_world.mThreadImpl

    local begin = thread_impl.updated_grid_worlds.begin
    local end_ = begin + thread_impl.chunk_update_count

    local count = thread_impl.chunk_update_count
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

        local rectangle = rect.Rectangle(start_x, start_y, end_x, end_y)
        -- rect_optimiser:submit(rectangle)
    end
    for i = 0, tonumber(thread_impl.world_update_params_count) - 1 do
        local wup = thread_impl.world_update_params.begin[i]
        local start_x = wup.update_region.top_left.x
        local start_y = wup.update_region.top_left.y
        local end_x = wup.update_region.bottom_right.x
        local end_y = wup.update_region.bottom_right.y

        local rectangle = rect.Rectangle(start_x, start_y, end_x, end_y)
        -- rect_optimiser:submit(rectangle)
    end

    local px, py = EntityGetTransform(ctx.my_player.entity)
    local ocx, ocy = math.floor(px / CHUNK_SIZE), math.floor(py / CHUNK_SIZE)
    for cx = ocx-1,ocx+1 do
        for cy = ocy-1,ocy+1 do
            local chunk_id = cx.." "..cy
            if initialized_chunks[chunk_id] == nil then
                local crect = rect.Rectangle(cx * CHUNK_SIZE, cy * CHUNK_SIZE, (cx+1) * CHUNK_SIZE, (cy+1) * CHUNK_SIZE)
                if DoesWorldExistAt(crect.left, crect.top, crect.right, crect.bottom) then
                    GamePrint("Sending chunk "..chunk_id)
                    initialized_chunks[chunk_id] = true
                    rect_optimiser:submit(crect)
                end
            end
        end
    end

    if GameGetFrameNum() % 1 == 0 then
        rect_optimiser:scan()

        for crect in rect.parts(rect_optimiser:iterate(), 256) do
            local area = nil
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
                    net.proxy_bin_send(KEY_WORLD_FRAME, str)
                end
            end
        end
        net.proxy_bin_send(KEY_WORLD_END, "")
        rect_optimiser:reset()
    end
end

local PixelRun_const_ptr = ffi.typeof("struct PixelRun const*")

function world_sync.handle_world_data(world_data)
    local grid_world = world_ffi.get_grid_world()
    for i, datum in ipairs(world_data) do
        -- GamePrint("Decoding world data "..i)
        local header = ffi.cast("struct EncodedAreaHeader const*", ffi.cast('char const*', datum))
        local runs = ffi.cast(PixelRun_const_ptr, ffi.cast("const char*", datum) + ffi.sizeof(world.EncodedAreaHeader))
        world.decode(grid_world, header, runs)
    end
end

return world_sync