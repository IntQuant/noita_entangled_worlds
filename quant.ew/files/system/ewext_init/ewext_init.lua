local ffi = require("ffi")

local module = {}

function module.on_world_initialized()
    local world_ffi = require("noitapatcher.nsew.world_ffi")
    local grid_world = world_ffi.get_grid_world()
    local chunk_map = grid_world.vtable.get_chunk_map(grid_world)
    grid_world = tonumber(ffi.cast("intptr_t", grid_world))
    chunk_map = tonumber(ffi.cast("intptr_t", chunk_map))
    ewext.init_particle_world_state(grid_world, chunk_map)
end

function module.on_local_player_spawn()
    local pix_p = ewext.get_pixel_pointer(0, 0)
    -- assert(pix_p ~= 0)
        -- assert(tonumber(ffi.cast("intptr_t", ppixel)) == pix_p)
    
end

return module