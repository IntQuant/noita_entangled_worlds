local util = dofile_once("mods/quant.ew/files/src/util.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local np = require("noitapatcher")

local module = {}

ModLuaFileAppend("data/scripts/director_helpers.lua", "mods/quant.ew/files/src/system/spawn_hooks/append/director_helpers.lua")

np.CrossCallAdd("ew_spawn_hook_pre", function(ent_path, x, y)
    if ctx.is_host then
        return true
    else
        return not module.entity_is_enemy(ent_path)
    end
end)

np.CrossCallAdd("ew_spawn_hook_post", function(ent)
    -- if not EntityHasTag(ent, "enemy") then
    --     EntityAddTag(ent, "ew_enemy_sync_extra")
    -- end
end)

local entity_is_enemy_cache = {}

function module.entity_is_enemy(ent_path)
    if entity_is_enemy_cache[ent_path] ~= nil then
        return entity_is_enemy_cache[ent_path]
    end

    print("Checking if this is an enemy:", ent_path)

    local ent = EntityLoad(ent_path) -- TODO: Just read xml maybe
    local res = EntityHasTag(ent, "enemy")
    EntityKill(ent)

    entity_is_enemy_cache[ent_path] = res
    return res
end

return module