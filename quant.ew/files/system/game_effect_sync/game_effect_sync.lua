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
    local filename = EntityGetFilename(entity)
    local list = {}
    for _, ent in ipairs(EntityGetAllChildren(entity) or {}) do
        -- Do not include disabled components here
        local com = EntityGetFirstComponent(ent, "GameEffectComponent")
        if com ~= nil then
            local name = ComponentGetValue2(com, "effect")
            if not IGNORE_EFFECTS[name] and filename ~= EntityGetFilename(ent) and not EntityHasTag(ent, "perk_entity") then
                table.insert(list, ent)
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

function effect_sync.get_sync_data(entity)
    local effects = effect_sync.get_ent_effects(entity)
    local sync_data = {}
    for _, effect in ipairs(effects) do
        local name = EntityGetFilename(effect)
        if name ~= nil and name ~= "" then --TODO serialize effects with no file
            local num = name_to_num(name)
            if num ~= -1 then
                table.insert(sync_data, num)
            else
                table.insert(sync_data, name)
            end
        end
    end
    return sync_data
end

function effect_sync.on_world_update()
    if GameGetFrameNum() % 30 == 9 then
        local sync_data = effect_sync.get_sync_data(ctx.my_player.entity)
        rpc.send_effects(sync_data)
    end
end

function effect_sync.remove_all_effects(entity)
    local effects = effect_sync.get_ent_effects(entity)
    for _, effect in ipairs(effects) do
        EntityKill(effect)
    end
end

function effect_sync.apply_effects(effects, entity)
    if not EntityGetIsAlive(entity) then
        return
    end
    local old_local_effects = effect_sync.get_ent_effects(entity)
    local effect_names = {}
    for _, effect in ipairs(effects) do
        local name
        local by_filenames = type(effect) ~= "number"
        if by_filenames then
            name = effect
        else
            name = constants.game_effects[effect]
        end
        table.insert(effect_names, name)
        for _, old_effect in ipairs(old_local_effects) do
            local old_com = EntityGetFirstComponentIncludingDisabled(old_effect, "GameEffectComponent")
            if old_com ~= nil then
                local old_name = EntityGetFilename(old_effect)
                if old_name == name then
                    local frames = ComponentGetValue2(old_com, "frames")
                    if frames ~= -1 then
                        ComponentSetValue2(old_com, "frames", 999999999)
                    end
                    goto continue
                end
            end
        end
        local ent = EntityLoad(name)
        EntityAddChild(entity, ent)
        local com = EntityGetFirstComponentIncludingDisabled(ent, "GameEffectComponent")
        if com ~= nil then
            ComponentSetValue2(com, "frames", 999999999)
        end
        ::continue::
    end

    local local_effects = effect_sync.get_ent_effects(entity)
    if #local_effects > #effects then
        for _, effect in ipairs(local_effects) do
            local local_name = EntityGetFilename(effect)
            local is_any = false
            for _, name in ipairs(effect_names) do
                if name == local_name then
                    is_any = true
                    break
                end
            end
            if not is_any then
                EntityKill(effect)
            end
        end
    end
end

function rpc.send_effects(effects)
    local entity = ctx.rpc_player_data.entity
    effect_sync.apply_effects(effects, entity)
end

return effect_sync