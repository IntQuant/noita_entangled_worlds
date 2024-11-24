local ffi = require("ffi")
local world_ffi = require("noitapatcher.nsew.world_ffi")

-- util.add_cross_call("make_ephemerial", ewext.make_ephemerial)

local initial_world_state_entity = nil

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

function module.on_world_update()
    if GameGetWorldStateEntity() ~= initial_world_state_entity then
        oh_another_world_state(GameGetWorldStateEntity())
        initial_world_state_entity = GameGetWorldStateEntity()
    end
    ewext.test_fn()
end

return module