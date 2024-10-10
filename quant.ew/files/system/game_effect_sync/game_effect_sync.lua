-- This module syncs effects (like being on fire) from clients to everyone else.

local rpc = net.new_rpc_namespace()

local effect_sync = {}

local IGNORE_EFFECTS = {
    POLYMORPH = true,
    POLYMORPH_RANDOM = true,
    POLYMORPH_CESSATION = true,
    POLYMORPH_UNSTABLE = true,
    NO_WAND_EDITING = true,
}

function effect_sync.get_ent_effects(entity, perks)
    local filename = EntityGetFilename(entity)
    local list = {}
    for _, ent in ipairs(EntityGetAllChildren(entity) or {}) do
        -- Do not include disabled components here
        if EntityHasTag(ent, "projectile") or EntityGetFirstComponentIncludingDisabled(ent, "LifetimeComponent") ~= nil then
            table.insert(list, ent)
        else
            local com = EntityGetFirstComponent(ent, "GameEffectComponent")
            if com ~= nil then
                local name = ComponentGetValue2(com, "effect")
                if not IGNORE_EFFECTS[name] and filename ~= EntityGetFilename(ent) and (not EntityHasTag(ent, "perk_entity") or perks) then
                    table.insert(list, ent)
                end
            end
        end
    end
    return list
end

local function name_to_num(name)
    for i, file_name in ipairs(constants.game_effects) do
        if file_name == name then
            return i
        end
    end
    return -1
end

function effect_sync.get_sync_data(entity, perks)
    local effects = effect_sync.get_ent_effects(entity, perks)
    local sync_data = {}
    for _, effect in ipairs(effects) do
        local name = EntityGetFilename(effect)
        if name ~= nil and name ~= "" then
            local num = name_to_num(name)
            if num ~= -1 then
                table.insert(sync_data, num)
            else
                table.insert(sync_data, name)
            end
        else
            table.insert(sync_data, np.SerializeEntity(effect))
        end
    end
    return sync_data
end

function effect_sync.on_world_update()
    if GameGetFrameNum() % 30 == 9 then
        local sync_data = effect_sync.get_sync_data(ctx.my_player.entity, false)
        rpc.send_effects(sync_data, false)
    end
end

function effect_sync.remove_all_effects(entity, perks)
    local effects = effect_sync.get_ent_effects(entity, perks)
    for _, effect in ipairs(effects) do
        EntityKill(effect)
    end
end

local function get_name(effect)
    local com = EntityGetFirstComponentIncludingDisabled(effect1, "GameEffectComponent")
    local name
    if com == nil then
        name = EntityGetFilename(effect)
    else
        name = ComponentGetValue2(com, "effect")
        if name == "CUSTOM" then
            name = ComponentGetValue2(com, "custom_effect_id")
        end
    end
    return name
end

local function remove_duplicates(effects)
    for i, effect1 in ipairs(effects) do
        local name1 = get_name(effect1)
        for j, effect2 in ipairs(effects) do
            if i ~= j and EntityGetIsAlive(effect1) and EntityGetIsAlive(effect2) then
                if name1 == get_name(effect2) then
                    if i < j then
                        EntityKill(effect1)
                    else
                        EntityKill(effect2)
                    end
                end
            end
        end
    end
end

function effect_sync.apply_effects(effects, entity, perks)
    if not EntityGetIsAlive(entity) then
        return
    end
    local old_local_effects = effect_sync.get_ent_effects(entity, perks)
    remove_duplicates(old_local_effects)
    local effect_names = {}
    for _, effect in ipairs(effects) do
        local name
        if type(effect) == "string" and (string.find(effect, "data/") == 1 or string.find(effect, "mods/") == 1) then
            name = effect
        elseif type(effect) == "number" then
            name = constants.game_effects[effect]
        else
            local serialized = EntityCreateNew()
            np.DeserializeEntity(serialized, effect)
            local com = EntityGetFirstComponentIncludingDisabled(serialized, "GameEffectComponent")
            local effect_name = get_name(serialized)
            for _, old_effect in ipairs(old_local_effects) do
                local old_com = EntityGetFirstComponentIncludingDisabled(old_effect, "GameEffectComponent")
                if effect_name == get_name(old_effect) then
                    if old_com ~= nil and ComponentGetValue2(old_com, "frames") ~= -1 then
                        ComponentSetValue2(old_com, "frames", 999999999)
                    end
                    EntityKill(serialized)
                    table.insert(effect_names, effect_name)
                    goto continue
                end
            end
            if com ~= nil and ComponentGetValue2(com, "frames") ~= -1 then
                ComponentSetValue2(com, "frames", 999999999)
            end
            EntityAddChild(entity, serialized)
            table.insert(effect_names, effect_name)
            goto continue
        end
        for _, old_effect in ipairs(old_local_effects) do
            local old_com = EntityGetFirstComponentIncludingDisabled(old_effect, "GameEffectComponent")
            local old_name = get_name(old_effect)
            if name == old_name then
                if old_com ~= nil and ComponentGetValue2(old_com, "frames") ~= -1 then
                    ComponentSetValue2(old_com, "frames", 999999999)
                end
                table.insert(effect_names, old_name)
                goto continue
            end
        end
        local ent = EntityLoad(name)
        if not EntityGetIsAlive(entity) then
            return
        end
        EntityAddChild(entity, ent)
        local com = EntityGetFirstComponentIncludingDisabled(ent, "GameEffectComponent")
        if com ~= nil and ComponentGetValue2(com, "frames") ~= -1 then
            ComponentSetValue2(com, "frames", 999999999)
        end
        table.insert(effect_names, get_name(ent))
        ::continue::
    end

    local local_effects = effect_sync.get_ent_effects(entity, perks)
    if #local_effects > #effect_names then
        for _, effect in ipairs(local_effects) do
            local local_name = get_name(effect)
            for _, name in ipairs(effect_names) do
                if name == local_name then
                    goto cont
                end
            end
            EntityKill(effect)
            ::cont::
        end
    end
    local is_on_fire = false
    for _, name in ipairs(effect_names) do
        if name == "ON_FIRE" then
            is_on_fire = true
        end
    end
    if not is_on_fire then
        local damage_model = EntityGetFirstComponentIncludingDisabled(entity, "DamageModelComponent")
        if damage_model ~= nil then
            ComponentSetValue2(damage_model, "mFireProbability", 0)
            ComponentSetValue2(damage_model, "mFireFramesLeft", 0)
        end
    end
end

function rpc.send_effects(effects, perks)
    local entity = ctx.rpc_player_data.entity
    effect_sync.apply_effects(effects, entity, perks)
end

return effect_sync