dofile_once("mods/quant.ew/NoitaPatcher/load.lua")
local np = require("noitapatcher")
package.cpath = package.cpath .. ";./mods/blob_guy/?.dll"
package.path = package.path .. ";./mods/blob_guy/?.lua"
local blob_guy = require("blob_guy")
local world_ffi = require("noitapatcher.nsew.world_ffi")
local ffi = require("ffi")
ModMaterialsFileAdd("mods/blob_guy/materials.xml")
local started = -1
function OnWorldPreUpdate()
    if started == -1 then
        return
    end
    if started > 0 then
        started = started - 1
        return
    end
    --local start_time = GameGetRealWorldTimeSinceStarted()
    blob_guy.update()
    --local end_time = GameGetRealWorldTimeSinceStarted()
    --local delta = (end_time - start_time) * 1000000
    --GamePrint(math.floor(delta + 0.5))
end
function OnWorldInitialized()
    started = 60
    local grid_world = world_ffi.get_grid_world()
    grid_world = tonumber(ffi.cast("intptr_t", grid_world))
    local material_list = tonumber(ffi.cast("intptr_t", world_ffi.get_material_ptr(0)))
    local world_info = np.GetWorldInfo()
    local construct_cell = tonumber(ffi.cast("intptr_t", world_info.construct_cell))
    local remove_cell = tonumber(ffi.cast("intptr_t", world_info.remove_cell))
    blob_guy.init_particle_world_state(grid_world, material_list, construct_cell, remove_cell)
end
