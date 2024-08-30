local util = dofile_once("mods/quant.ew/files/core/util.lua")
local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local np = require("noitapatcher")

local module = {}

ModLuaFileAppend("data/scripts/director_helpers.lua",
    "mods/quant.ew/files/system/spawn_hooks/append/director_helpers.lua")
ModLuaFileAppend("data/scripts/item_spawnlists.lua", "mods/quant.ew/files/system/spawn_hooks/append/item_spawnlist.lua")

local exclude = {}
exclude["data/entities/items/pickup/perk.xml"] = true
exclude["data/entities/items/pickup/spell_refresh.xml"] = true
exclude["data/entities/items/pickup/heart.xml"] = true
exclude["data/entities/items/pickup/heart_better.xml"] = true
exclude["data/entities/items/pickup/heart_evil.xml"] = true
exclude["data/entities/items/pickup/heart_fullhp.xml"] = true
exclude["data/entities/items/pickup/heart_fullhp_temple.xml"] = true
exclude["data/entities/items/pickup/perk_reroll.xml"] = true

-- This entity needs to be synced by item_sync
local function is_sync_item(ent_path)
    -- No item needs to be synced when this option is off.
    if not ctx.proxy_opt.item_dedup then
        return false
    end
    if exclude[ent_path] then
        return false
    end
    local start = "data/entities/items/"
    if string.sub(ent_path, 1, #start) == start then
        return true
    end
    if ent_path == "data/entities/misc/custom_cards/action.xml" then
        return true
    end
    return false
end

np.CrossCallAdd("ew_spawn_hook_pre", function(ent_path, x, y)
    if ctx.is_host then
        if is_sync_item(ent_path) then
            local ent_id = EntityLoad(ent_path, x, y)
            ctx.cap.item_sync.globalize(ent_id, false)
            return ent_id
        else
            return true
        end
    else
        if is_sync_item(ent_path) then
            return false
        else
            return not module.entity_is_synced(ent_path)
        end
    end
end)

np.CrossCallAdd("ew_action_spawn_hook_pre", function()
    return (not ctx.proxy_opt.item_dedup) or ctx.is_host
end)

np.CrossCallAdd("ew_action_spawn_hook", function(eid)
    ctx.cap.item_sync.globalize(eid, false)
end)

-- Called after entity was loaded.
-- Might be useless in some cases, as entity was already despawned/serialized due to CameraBoundComponent.
np.CrossCallAdd("ew_spawn_hook_post", function(ent_path, ent)

end)

local entity_is_enemy_cache = {}

function module.entity_is_synced(ent_path)
    if entity_is_enemy_cache[ent_path] ~= nil then
        return entity_is_enemy_cache[ent_path]
    end

    print("Checking if this is an enemy: " .. ent_path)

    local tags = util.load_ents_tags(ent_path)

    local res = constants.phys_sync_allowed[ent_path] or table.contains(tags, "enemy")

    entity_is_enemy_cache[ent_path] = res
    return res
end

return module
