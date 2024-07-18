local util = dofile_once("mods/quant.ew/files/src/util.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local np = require("noitapatcher")

local module = {}

ModLuaFileAppend("data/scripts/director_helpers.lua", "mods/quant.ew/files/src/system/spawn_hooks/append/director_helpers.lua")
ModLuaFileAppend("data/scripts/item_spawnlists.lua", "mods/quant.ew/files/src/system/spawn_hooks/append/item_spawnlist.lua")

-- This entity needs to be synced by item_sync
local function is_sync_item(ent_path)
    -- No item needs to be synced when this option is off.
    if not ctx.proxy_opt.item_dedup then
        return false
    end
    local start = "data/entities/items/"
    if string.sub(ent_path, 1, #start) ~= start then
        return false
    end
    return true
end

np.CrossCallAdd("ew_spawn_hook_pre", function(ent_path, x, y)
    if ctx.is_host then
        if is_sync_item(ent_path) then
            local ent_id = EntityLoad(ent_path, x, y)
            ctx.cap.item_sync.globalize(ent_id, true)
            return false
        else
            return true
        end
    else
        if is_sync_item(ent_path) then
            return false
        else
            return not module.entity_is_enemy(ent_path)
        end
    end
end)

-- Called after entity was loaded.
-- Might be useless in some cases, as entity was already despawned/serialized due to CameraBoundComponent.
np.CrossCallAdd("ew_spawn_hook_post", function(ent_path, ent)
    
end)

local entity_is_enemy_cache = {}

function module.entity_is_enemy(ent_path)
    if entity_is_enemy_cache[ent_path] ~= nil then
        return entity_is_enemy_cache[ent_path]
    end

    print("Checking if this is an enemy: "..ent_path)

    local ent = EntityLoad(ent_path) -- TODO: Just read xml maybe
    local res = EntityHasTag(ent, "enemy")
    EntityKill(ent)

    entity_is_enemy_cache[ent_path] = res
    return res
end

return module
