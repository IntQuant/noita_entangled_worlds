local ffi = require("ffi")
local world_ffi = require("noitapatcher.nsew.world_ffi")

-- util.add_cross_call("make_ephemerial", ewext.make_ephemerial)

local initial_world_state_entity

local module = {}

function module.on_world_initialized()
    initial_world_state_entity = GameGetWorldStateEntity()
    ewext.on_world_initialized()
    local grid_world = world_ffi.get_grid_world()
    local chunk_map = grid_world.vtable.get_chunk_map(grid_world)
    grid_world = tonumber(ffi.cast("intptr_t", grid_world))
    chunk_map = tonumber(ffi.cast("intptr_t", chunk_map))
    local material_list = tonumber(ffi.cast("intptr_t", world_ffi.get_material_ptr(0)))
    ewext.init_particle_world_state(grid_world, chunk_map, material_list)
    ewext.module_on_world_init()
end

local function oh_another_world_state(entity)
    print("Another world state: "..entity)
    GamePrint("Another World State Entity detected")
    GamePrint("Do a 'mods > restart with enabled mods' to avoid a crash")
    util.make_ephemerial(entity)
end

function module.on_local_player_spawn()
    initial_world_state_entity = GameGetWorldStateEntity()
    for _, ent in ipairs(EntityGetWithTag("world_state")) do
        if ent ~= GameGetWorldStateEntity() then
            oh_another_world_state(ent)
        end
    end
    EntitySetTransform(GameGetWorldStateEntity(), 0, 0)
end

local function fw_button(label)
    return imgui.Button(label, imgui.GetWindowWidth() - 15, 20)
end

local function bench_fn_lua()
    local start = GameGetRealWorldTimeSinceStarted()
    for i=1,10000 do
        local player = EntityGetClosestWithTag(0, 0, "player_unit")
        EntitySetTransform(player, 0, 0, 0, 1, 1)
    end
    local elapsed = GameGetRealWorldTimeSinceStarted() - start
    GamePrint(elapsed*1000000)
end

function module.on_draw_debug_window(imgui)
    if imgui.CollapsingHeader("ewext") then
        if fw_button("test_fn") then
            ewext.test_fn()
        end
        if fw_button("bench") then
            ewext.bench_fn()
            bench_fn_lua()
        end
    end
end

function module.on_world_update()
    if GameGetWorldStateEntity() ~= initial_world_state_entity then
        oh_another_world_state(GameGetWorldStateEntity())
        initial_world_state_entity = GameGetWorldStateEntity()
    end
    ewext.module_on_world_update()
end

return module