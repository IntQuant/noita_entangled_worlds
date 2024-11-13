local ffi = require("ffi")
local world_ffi = require("noitapatcher.nsew.world_ffi")

-- np.CrossCallAdd("make_ephemerial", ewext.make_ephemerial)

local initial_world_state_entity = nil

local module = {}

function module.on_world_initialized()
    initial_world_state_entity = GameGetWorldStateEntity()
    ewext.save_world_state()
    local grid_world = world_ffi.get_grid_world()
    local chunk_map = grid_world.vtable.get_chunk_map(grid_world)
    grid_world = tonumber(ffi.cast("intptr_t", grid_world))
    chunk_map = tonumber(ffi.cast("intptr_t", chunk_map))
    local material_list = tonumber(ffi.cast("intptr_t", world_ffi.get_material_ptr(0)))
    ewext.init_particle_world_state(grid_world, chunk_map, material_list)
end

function module.on_local_player_spawn()
       
end

function module.on_world_update()
    if GameGetWorldStateEntity() ~= initial_world_state_entity then
        GamePrint("Whoops WSE is different "..GameGetWorldStateEntity().." "..initial_world_state_entity)
        ewext.make_ephemerial(GameGetWorldStateEntity())
        -- EntityKill(GameGetWorldStateEntity())
        ewext.load_world_state()
    end
end

return module
