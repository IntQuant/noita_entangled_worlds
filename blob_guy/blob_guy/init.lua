dofile_once("mods/quant.ew/NoitaPatcher/load.lua")
local np = require("noitapatcher")
package.cpath = package.cpath .. ";./mods/blob_guy/?.dll"
package.path = package.path .. ";./mods/blob_guy/?.lua"
local blob_guy = require("blob_guy")
local world_ffi = require("noitapatcher.nsew.world_ffi")
local ffi = require("ffi")
ModMaterialsFileAdd("mods/blob_guy/materials.xml")
local started = -1
local times = 0
local start = 10
local times_len = start
function OnWorldPreUpdate()
    if started == -1 then
        return
    end
    if started > 0 then
        started = started - 1
        return
    end
    local start_time = GameGetRealWorldTimeSinceStarted()
    blob_guy.update()
    local end_time = GameGetRealWorldTimeSinceStarted()
    local delta = (end_time - start_time) * 1000000
    times = times + delta
    times_len = times_len - 1
    if times_len == 0 then
        times_len = start
        GamePrint(math.floor(times / start + 0.5))
        times = 0
    end
end
function OnWorldInitialized()
    started = 60
    local grid_world = world_ffi.get_grid_world()
    grid_world = tonumber(ffi.cast("intptr_t", grid_world))
    blob_guy.init_particle_world_state(grid_world)
end
