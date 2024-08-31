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

local iter_fast = false

local iter_slow = 0

function round(x)
    return x >= 0 and math.floor(x + 0.5) or math.ceil(x - 0.5)
end

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
    local int = 3 -- ctx.proxy_opt.world_sync_interval
    if GameGetFrameNum() % int == 0 then
        for cx = ocx - 1, ocx do
            for cy = ocy - 1, ocy do
                send_chunks(cx, cy, chunk_map)
            end
        end
        net.proxy_bin_send(KEY_WORLD_END, string.char(priority) .. pos_data)
    elseif GameGetFrameNum() % (int * 2) == 1 then
        local nx = ocx
        if iter_fast then
            nx = nx - 2
        end
        for cx = nx, nx + 1 do
            for cy = ocy - 2, ocy + 1 do
                if cx < ocx - 1 or cx > ocx or cy < ocy - 1 or cy > ocy then
                    send_chunks(cx, cy, chunk_map)
                end
            end
        end
        net.proxy_bin_send(KEY_WORLD_END, string.char(priority + 1))
        iter_fast = not iter_fast
    elseif priority == 0 and GameGetFrameNum() % (int * 4) == 4 then
        local nx = ocx
        if iter_slow == 1 or iter_slow == 2 then
            nx = nx - 3
        end
        for cx = nx, nx + 2 do
            local ny = ocy
            if iter_slow == 0 or iter_slow == 1 then
                ny = ny - 3
            end
            for cy = ny, ny + 2  do
                if cx < ocx - 2 or cx > ocx + 1 or cy < ocy - 2 or cy > ocy + 1 then
                    send_chunks(cx, cy, chunk_map)
                end
            end
        end
        net.proxy_bin_send(KEY_WORLD_END, string.char(priority + 2))
        iter_slow = iter_slow + 1
        if iter_slow == 4 then
            iter_slow = 0
        end
    end
end


function world_sync.on_world_update()
    local cx, cy = GameGetCameraPos()
    cx, cy = round(cx / CHUNK_SIZE), round(cy / CHUNK_SIZE)
    local player_data = ctx.my_player
    if not EntityGetIsAlive(player_data.entity) then
        return
    end
    local px, py = EntityGetTransform(player_data.entity)
    -- Original Chunk x/y
    local ocx, ocy = round(px / CHUNK_SIZE), round(py / CHUNK_SIZE)
    local pos_data = ocx..":"..ocy..":"..cx..":"..cy
    if math.abs(cx - ocx) > 2 or math.abs(cy - ocy) > 2 then
        if iter_cam then
            get_all_chunks(cx, cy, pos_data, 16)
        else
            get_all_chunks(ocx, ocy, pos_data, 32)
        end
        local int = 3 -- ctx.proxy_opt.world_sync_interval
        if GameGetFrameNum() % (int * 4) == 0 then
            iter_cam = not iter_cam
        end
    else
        get_all_chunks(ocx, ocy, pos_data, 0)
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