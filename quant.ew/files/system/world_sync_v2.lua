local world_ffi = require("noitapatcher.nsew.world_ffi")
local world = require("noitapatcher.nsew.world")
local rect = require("noitapatcher.nsew.rect")
local ffi = require("ffi")

local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/core/player_fns.lua")

-- local rpc = net.new_rpc_namespace()

local rect_optimiser = rect.Optimiser_new()
local encoded_area = world.EncodedArea()

local world_sync = {}

local KEY_WORLD_FRAME = 0
local KEY_WORLD_END = 1

local CHUNK_SIZE = 128

local iter = 0

function world_sync.on_world_update()

    local grid_world = world_ffi.get_grid_world()
    local chunk_map = grid_world.vtable.get_chunk_map(grid_world)
    local thread_impl = grid_world.mThreadImpl

    if GameGetFrameNum() % ctx.proxy_opt.world_sync_interval == 0 then
        local player_data = ctx.my_player
        if not EntityGetIsAlive(player_data.entity) then
            return
        end
        local px, py = EntityGetTransform(player_data.entity)
        -- Original Chunk x/y
        local ocx, ocy = math.floor(px / CHUNK_SIZE), math.floor(py / CHUNK_SIZE)

        local cx = ocx - 2 + iter
        for cy = ocy-2,ocy+2 do
            local crect = rect.Rectangle(cx * CHUNK_SIZE, cy * CHUNK_SIZE, (cx+1) * CHUNK_SIZE, (cy+1) * CHUNK_SIZE)
            if DoesWorldExistAt(crect.left, crect.top, crect.right, crect.bottom) then
                local area = world.encode_area(chunk_map, crect.left, crect.top, crect.right, crect.bottom, encoded_area)
                if area ~= nil then
                    if ctx.proxy_opt.debug then
                        GameCreateSpriteForXFrames("mods/quant.ew/files/debug/box_128x128.png", crect.left+64, crect.top + 64, true, 0, 0, 11, true)
                    end
                    local str = ffi.string(area, world.encoded_size(area))
                    net.proxy_bin_send(KEY_WORLD_FRAME, str)
                end
            end
        end
        iter = iter + 1
        if iter > 5 then
            iter = 0
        end

        net.proxy_bin_send(KEY_WORLD_END, "")
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
