local ffi = require("ffi")
local world_ffi = require("noitapatcher.nsew.world_ffi")

local module = {}

function module.on_world_initialized()
    local grid_world = world_ffi.get_grid_world()
    local chunk_map = grid_world.vtable.get_chunk_map(grid_world)
    grid_world = tonumber(ffi.cast("intptr_t", grid_world))
    chunk_map = tonumber(ffi.cast("intptr_t", chunk_map))
    local material_list = tonumber(ffi.cast("intptr_t", world_ffi.get_material_ptr(0)))
    ewext.init_particle_world_state(grid_world, chunk_map, material_list)
end

function module.on_local_player_spawn()
       
end

return module