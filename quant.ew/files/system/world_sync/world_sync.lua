local world_ffi = require("noitapatcher.nsew.world_ffi")
local world = dofile_once("mods/quant.ew/files/system/world_sync/world.lua")
local rect = require("noitapatcher.nsew.rect")
local ffi = require("ffi")

local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")

-- local rpc = net.new_rpc_namespace()

--local rect_optimiser = rect.Optimiser_new()
local encoded_area = world.EncodedArea()

local world_sync = {}

local KEY_WORLD_FRAME = 0
local KEY_WORLD_END = 1

local CHUNK_SIZE = 128

local iter_cam = false

local iter_fast = 0

local iter_slow = 0

local iter_slow_2 = 0

function world_sync.on_world_initialized()
    local c = 0
    while true do
        local name = CellFactory_GetName(c)
        if name == "unknown" then
            break
        end
        c = c + 1
    end
    c = c - 1
    print("Last material id: "..c)
    world.last_material_id = c
end

local function send_chunks(cx, cy, chunk_map)
    local chx, chy = cx * CHUNK_SIZE, cy * CHUNK_SIZE
    local crect = rect.Rectangle(chx, chy, chx + CHUNK_SIZE, chy + CHUNK_SIZE)
    if DoesWorldExistAt(crect.left, crect.top, crect.right, crect.bottom) then
        local area = world.encode_area(chunk_map, crect.left, crect.top, crect.right, crect.bottom, encoded_area)
        if area ~= nil then
            --if ctx.proxy_opt.debug then
            --     GameCreateSpriteForXFrames("mods/quant.ew/files/resource/debug/box_128x128.png", crect.left+64, crect.top + 64, true, 0, 0, 11, true)
            --end
            local str = ffi.string(area, world.encoded_size(area))
            net.proxy_bin_send(KEY_WORLD_FRAME, str)
        end
    end
end

local function get_all_chunks(ocx, ocy, pos_data, priority)
    local grid_world = world_ffi.get_grid_world()
    local chunk_map = grid_world.vtable.get_chunk_map(grid_world)
    --local thread_impl = grid_world.mThreadImpl
    local int = 4 -- ctx.proxy_opt.world_sync_interval
    if GameGetFrameNum() % int == 0 then
        send_chunks(ocx, ocy, chunk_map)
        net.proxy_bin_send(KEY_WORLD_END, string.char(priority))
    elseif GameGetFrameNum() % int == 2 then
        if iter_fast == 0 then
            send_chunks(ocx + 1, ocy, chunk_map)
            send_chunks(ocx + 1, ocy + 1, chunk_map)
        elseif iter_fast == 1 then
            send_chunks(ocx, ocy + 1, chunk_map)
            send_chunks(ocx - 1, ocy + 1, chunk_map)
        elseif iter_fast == 2 then
            send_chunks(ocx - 1, ocy, chunk_map)
            send_chunks(ocx - 1, ocy - 1, chunk_map)
        else
            send_chunks(ocx, ocy - 1, chunk_map)
            send_chunks(ocx + 1, ocy - 1, chunk_map)
        end
        net.proxy_bin_send(KEY_WORLD_END, string.char(math.min(priority + 1, 16)))
        iter_fast = iter_fast + 1
        if iter_fast == 4 then
            iter_fast = 0
        end
    elseif GameGetFrameNum() % (int * 4) == 3 then
        if iter_slow == 0 then
            send_chunks(ocx + 2, ocy - 1, chunk_map)
            send_chunks(ocx + 2, ocy, chunk_map)
            send_chunks(ocx + 2, ocy + 1, chunk_map)
            send_chunks(ocx + 2, ocy + 2, chunk_map)
        elseif iter_slow == 1 then
            send_chunks(ocx + 1, ocy + 2, chunk_map)
            send_chunks(ocx, ocy + 2, chunk_map)
            send_chunks(ocx - 1, ocy + 2, chunk_map)
            send_chunks(ocx - 2, ocy + 2, chunk_map)
        elseif iter_slow == 2 then
            send_chunks(ocx - 2, ocy + 1, chunk_map)
            send_chunks(ocx - 2, ocy, chunk_map)
            send_chunks(ocx - 2, ocy - 1, chunk_map)
            send_chunks(ocx - 2, ocy - 2, chunk_map)
        else
            send_chunks(ocx - 1, ocy - 2, chunk_map)
            send_chunks(ocx, ocy - 2, chunk_map)
            send_chunks(ocx + 1, ocy - 2, chunk_map)
            send_chunks(ocx + 2, ocy - 2, chunk_map)
        end
        net.proxy_bin_send(KEY_WORLD_END, string.char(math.min(priority + 2, 16)) .. pos_data)
        iter_slow = iter_slow + 1
        if iter_slow == 4 then
            iter_slow = 0
        end
    elseif priority == 0 and GameGetFrameNum() % (int * 3) == 1 then
        if iter_slow_2 == 0 then
            send_chunks(ocx + 3, ocy, chunk_map)
            send_chunks(ocx + 3, ocy + 1, chunk_map)
            send_chunks(ocx + 3, ocy + 2, chunk_map)
            send_chunks(ocx + 3, ocy + 3, chunk_map)
        elseif iter_slow_2 == 1 then
            send_chunks(ocx + 2, ocy + 3, chunk_map)
            send_chunks(ocx + 1, ocy + 3, chunk_map)
            send_chunks(ocx, ocy + 3, chunk_map)
            send_chunks(ocx - 1, ocy + 3, chunk_map)
        elseif iter_slow_2 == 2 then
            send_chunks(ocx - 2, ocy + 3, chunk_map)
            send_chunks(ocx - 3, ocy + 3, chunk_map)
            send_chunks(ocx - 3, ocy + 2, chunk_map)
            send_chunks(ocx - 3, ocy + 1, chunk_map)
        elseif iter_slow_2 == 3 then
            send_chunks(ocx - 3, ocy, chunk_map)
            send_chunks(ocx - 3, ocy - 1, chunk_map)
            send_chunks(ocx - 3, ocy - 2, chunk_map)
            send_chunks(ocx - 3, ocy - 3, chunk_map)
        elseif iter_slow_2 == 4 then
            send_chunks(ocx - 2, ocy - 3, chunk_map)
            send_chunks(ocx - 1, ocy - 3, chunk_map)
            send_chunks(ocx, ocy - 3, chunk_map)
            send_chunks(ocx + 1, ocy - 3, chunk_map)
        else
            send_chunks(ocx + 2, ocy - 3, chunk_map)
            send_chunks(ocx + 3, ocy - 3, chunk_map)
            send_chunks(ocx + 3, ocy - 2, chunk_map)
            send_chunks(ocx + 3, ocy - 1, chunk_map)
        end
        net.proxy_bin_send(KEY_WORLD_END, string.char(math.min(priority + 2, 16)))
        iter_slow_2 = iter_slow_2 + 1
        if iter_slow_2 == 6 then
            iter_slow_2 = 0
        end
    end
end

function world_sync.on_world_update()
    local cx, cy = GameGetCameraPos()
    cx, cy = math.floor(cx / CHUNK_SIZE), math.floor(cy / CHUNK_SIZE)
    local player_data = ctx.my_player
    if not EntityGetIsAlive(player_data.entity) then
        return
    end
    local px, py = EntityGetTransform(player_data.entity)
    -- Original Chunk x/y
    local ocx, ocy = math.floor(px / CHUNK_SIZE), math.floor(py / CHUNK_SIZE)
    local pos_data = ocx..":"..ocy..":"..cx..":"..cy
    if math.abs(cx - ocx) > 2 or math.abs(cy - ocy) > 2 then
        if GameGetFrameNum() % 3 ~= 2 then
            get_all_chunks(cx, cy, pos_data, 16)
        else
            get_all_chunks(ocx, ocy, pos_data, 16)
        end
    else
        local pri = 0
        if EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
            pri = 16
        end
        get_all_chunks(ocx, ocy, pos_data, pri)
    end
end

local PixelRun_const_ptr = ffi.typeof("struct PixelRun const*")

function world_sync.handle_world_data(datum)
    local grid_world = world_ffi.get_grid_world()
    local header = ffi.cast("struct EncodedAreaHeader const*", ffi.cast('char const*', datum))
    local runs = ffi.cast(PixelRun_const_ptr, ffi.cast("const char*", datum) + ffi.sizeof(world.EncodedAreaHeader))
    world.decode(grid_world, header, runs)
end

net.net_handling.proxy[0] = function(_, value)
    world_sync.handle_world_data(value)
end

return world_sync