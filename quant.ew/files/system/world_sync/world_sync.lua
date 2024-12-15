local world_ffi = require("noitapatcher.nsew.world_ffi")
local world = dofile_once("mods/quant.ew/files/system/world_sync/world.lua")
local rect = require("noitapatcher.nsew.rect")
local ffi = require("ffi")

-- local rpc = net.new_rpc_namespace()

--local rect_optimiser = rect.Optimiser_new()
local encoded_area = world.EncodedArea()

local world_sync = {}

local KEY_WORLD_FRAME = 0
local KEY_WORLD_END = 1

local CHUNK_SIZE = 128

local iter_fast = 0

local iter_slow = 0

local iter_slow_2 = 0

--[[local function do_benchmark()
    local world_ffi = require("noitapatcher.nsew.world_ffi")
    local grid_world = world_ffi.get_grid_world()
    local chunk_map = grid_world.vtable.get_chunk_map(grid_world)
    local start = GameGetRealWorldTimeSinceStarted()
    local iters = 10000
    for i=1, iters do
        world.encode_area(chunk_map, 0, 0, 128, 128, encode_area)
        -- world_ffi.get_cell(chunk_map, 0, 0)
    end
    local end_time = GameGetRealWorldTimeSinceStarted()
    local elapsed = (end_time - start) * 1000 * 1000 * 1000 / (iters * 128 * 128)
    print("Benchmark:", elapsed, "ns/pixel")
end]]

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
    -- do_benchmark()
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
local int = 4 -- ctx.proxy_opt.world_sync_interval

local function get_all_chunks(ocx, ocy, pos_data, priority, give_0)
    local grid_world = world_ffi.get_grid_world()
    local chunk_map = grid_world.vtable.get_chunk_map(grid_world)
    --local thread_impl = grid_world.mThreadImpl
    if GameGetFrameNum() % int == 0 then
        send_chunks(ocx, ocy, chunk_map)
        local pri = priority
        if give_0 then
            pri = 0
        end
        net.proxy_bin_send(KEY_WORLD_END, string.char(pri)..pos_data)
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
        net.proxy_bin_send(KEY_WORLD_END, string.char(math.min(priority + 1, 16))..pos_data)
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
    elseif (priority == 0 and not GameHasFlagRun("ending_game_completed")) and GameGetFrameNum() % (int * 3) == 1 then
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
        net.proxy_bin_send(KEY_WORLD_END, string.char(math.min(priority + 2, 16))..pos_data)
        iter_slow_2 = iter_slow_2 + 1
        if iter_slow_2 == 6 then
            iter_slow_2 = 0
        end
    end
end

function world_sync.on_world_update()
    if ctx.run_ended then
        return
    end
    int = math.floor(tonumber(ModSettingGet("quant.ew.world_sync")))
    local cx, cy = GameGetCameraPos()
    cx, cy = math.floor(cx / CHUNK_SIZE), math.floor(cy / CHUNK_SIZE)
    local player_data = ctx.my_player
    if not EntityGetIsAlive(player_data.entity) then
        return
    end
    local px, py = EntityGetTransform(player_data.entity)
    -- Original Chunk x/y
    local ocx, ocy = math.floor(px / CHUNK_SIZE), math.floor(py / CHUNK_SIZE)
    local n = 0
    if EntityHasTag(ctx.my_player.entity, "ew_notplayer") or GameHasFlagRun("ending_game_completed") then
        n = 1
    end
    local pos_data
    if GameGetFrameNum() % int ~= 0 and GameGetFrameNum() % (int * 4) == 3 then
        pos_data = ocx..":"..ocy..":"..cx..":"..cy..":"..n ..":"..ctx.proxy_opt.world_num
    else
        pos_data = ctx.proxy_opt.world_num
    end
    if math.abs(cx - ocx) > 2 or math.abs(cy - ocy) > 2 then
        if GameGetFrameNum() % 3 ~= 2 then
            get_all_chunks(cx, cy, pos_data, 16, false)
        else
            get_all_chunks(ocx, ocy, pos_data, 16, true)
        end
    else
        local pri = 0
        if EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
            pri = 16
        end
        get_all_chunks(ocx, ocy, pos_data, pri, true)
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