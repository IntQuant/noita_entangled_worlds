-- This module syncs effects (like being on fire) from clients to everyone else.

local rpc = net.new_rpc_namespace()

local effect_sync = {}

local IGNORE_EFFECTS = {
    POLYMORPH = true,
    POLYMORPH_RANDOM = true,
    POLYMORPH_CESSATION = true,
    POLYMORPH_UNSTABLE = true,
}

function effect_sync.get_ent_effects(entity)
    local list = {}
    for _, ent in ipairs(EntityGetAllChildren(entity) or {}) do
        -- Do not include disabled components here
        local com = EntityGetFirstComponent(ent, "GameEffectComponent")
        if com ~= nil then
            local name = ComponentGetValue2(com, "effect")
            if not IGNORE_EFFECTS[name] then
                table.insert(list, ent)
            end
            -- GamePrint("eff "..name)
        end
    end
    return list
end

local local_by_remote_id = {}

function effect_sync.on_world_update()
    if GameGetFrameNum() % 30 == 9 then
        local effects = effect_sync.get_ent_effects(ctx.my_player.entity)
        local sync_data = {}
        for _, effect in ipairs(effects) do
            table.insert(sync_data, {effect, EntityGetFilename(effect)})
        end
        rpc.send_effects(sync_data)
    end
    -- Cleanup
    if GameGetFrameNum() % 120 == 9 then
        for rem_id, loc_id in pairs(local_by_remote_id) do
            if not EntityGetIsAlive(loc_id) then
                local_by_remote_id[rem_id] = nil
            end
        end
    end
end

function effect_sync.remove_all_effects(entity)
    local effects = effect_sync.get_ent_effects(entity)
    for _, effect in ipairs(effects) do
        EntityKill(effect)
    end
end

function rpc.send_effects(effects)
    local entity = ctx.rpc_player_data.entity
    local confirmed_effects = {}
    for _, effect in ipairs(effects) do
        local effect_remote_id = effect[1]
        if local_by_remote_id[effect_remote_id] == nil or not EntityGetIsAlive(local_by_remote_id[effect_remote_id]) then
            local ent = EntityLoad(effect[2])
            EntityAddChild(entity, ent)
            -- GamePrint("Replicating "..effect_remote_id.." as "..ent)
            local_by_remote_id[effect_remote_id] = ent
            local com = EntityGetFirstComponentIncludingDisabled(ent, "GameEffectComponent")
            if com ~= nil then
                ComponentSetValue2(com, "frames", 999999999)
            end
        end
        confirmed_effects[local_by_remote_id[effect_remote_id]] = true
    end

    local local_effects = effect_sync.get_ent_effects(entity)
    for _, effect in ipairs(local_effects) do
        if not confirmed_effects[effect] then
            -- GamePrint("Removing "..effect)
            EntityKill(effect)
        end
    end
end

return effect_sync