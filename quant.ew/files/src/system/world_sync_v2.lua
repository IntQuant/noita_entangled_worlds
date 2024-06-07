local world_ffi = require("noitapatcher.nsew.world_ffi")
local world = require("noitapatcher.nsew.world")
local rect = require("noitapatcher.nsew.rect")
local ffi = require("ffi")

local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")

-- local rpc = net.new_rpc_namespace()

local rect_optimiser = rect.Optimiser_new()
local encoded_area = world.EncodedArea()

local world_sync = {}

local KEY_WORLD_FRAME = 0
local KEY_WORLD_END = 1

local CHUNK_SIZE = 128

local function chunk_producer()
    local initialized_chunks = {}
    local sent_anything = false

    for _, player_data in pairs(ctx.players) do
        local px, py = EntityGetTransform(player_data.entity)
        local ocx, ocy = math.floor(px / CHUNK_SIZE), math.floor(py / CHUNK_SIZE)

        for cx = ocx-1,ocx+1 do
            for cy = ocy-1,ocy+1 do
                local chunk_id = cx.." "..cy
                if initialized_chunks[chunk_id] == nil then
                    local crect = rect.Rectangle(cx * CHUNK_SIZE, cy * CHUNK_SIZE, (cx+1) * CHUNK_SIZE, (cy+1) * CHUNK_SIZE)
                    if DoesWorldExistAt(crect.left, crect.top, crect.right, crect.bottom) then
                        -- GamePrint("Sending chunk "..chunk_id)
                        initialized_chunks[chunk_id] = true
                        coroutine.yield(crect)
                    end
                end
            end
        end
    end
end

local producer_coro = nil

function world_sync.on_world_update_host()

    local grid_world = world_ffi.get_grid_world()
    local chunk_map = grid_world.vtable.get_chunk_map(grid_world)
    local thread_impl = grid_world.mThreadImpl

    local sent_anything = false

    if producer_coro == nil then
        producer_coro = coroutine.wrap(chunk_producer)
    end

    if GameGetFrameNum() % 4 == 0 then
        local crect = producer_coro()

        if crect == nil then
            producer_coro = nil
            return
        end

        local area = world.encode_area(chunk_map, crect.left, crect.top, crect.right, crect.bottom, encoded_area)
        if area ~= nil then
            local str = ffi.string(area, world.encoded_size(area))
            net.proxy_bin_send(KEY_WORLD_FRAME, str)
            sent_anything = true
        end

        if sent_anything then
            net.proxy_bin_send(KEY_WORLD_END, "")
        end
    end
end

local PixelRun_const_ptr = ffi.typeof("struct PixelRun const*")

function world_sync.handle_world_data(datum)
    local grid_world = world_ffi.get_grid_world()
        -- GamePrint("Decoding world data "..i)
    local header = ffi.cast("struct EncodedAreaHeader const*", ffi.cast('char const*', datum))
    local runs = ffi.cast(PixelRun_const_ptr, ffi.cast("const char*", datum) + ffi.sizeof(world.EncodedAreaHeader))
    world.decode(grid_world, header, runs)
end

net.net_handling.proxy[0] = function(_, value)
    world_sync.handle_world_data(value)
end

return world_sync