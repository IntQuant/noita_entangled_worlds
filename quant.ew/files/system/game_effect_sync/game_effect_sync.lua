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

local local_by_remote_id = {}

function effect_sync.on_world_update()
    if GameGetFrameNum() % 30 == 9 then
        local effects = effect_sync.get_ent_effects(ctx.my_player.entity)
        local sync_data = {}
        local by_filenames = false
        for _, effect in ipairs(effects) do
            local num = name_to_num(EntityGetFilename(effect))
            if num ~= -1 then
                table.insert(sync_data, {effect, num})
            else
                by_filenames = true
                break
            end
        end
        if by_filenames then
            for _, effect in ipairs(effects) do
                table.insert(sync_data, {effect, EntityGetFilename(effect)})
            end
            rpc.send_effects(sync_data, true)
        else
            rpc.send_effects(sync_data, false)
        end
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

function rpc.send_effects(effects, by_filenames)
    local entity = ctx.rpc_player_data.entity
    if not EntityGetIsAlive(entity) then
        return
    end
    local confirmed_effects = {}
    local old_local_effects = effect_sync.get_ent_effects(entity)
    for _, effect in ipairs(effects) do
        local effect_remote_id = effect[1]
        if local_by_remote_id[effect_remote_id] == nil or not EntityGetIsAlive(local_by_remote_id[effect_remote_id]) or not EntityGetIsAlive(entity) then
            for _, old_effect in ipairs(old_local_effects) do
                local old_com = EntityGetFirstComponentIncludingDisabled(old_effect, "GameEffectComponent")
                if old_com ~= nil then
                    local is_same
                    if by_filenames then
                        is_same = effect[2] == EntityGetFilename(old_effect)
                    else
                        is_same = constants.game_effects[effect[2]] == EntityGetFilename(old_effect)
                    end
                    if is_same then
                        ComponentSetValue2(old_com, "frames", 999999999)
                        local_by_remote_id[effect_remote_id] = old_effect
                        goto continue
                    end
                end
            end
            local ent
            if by_filenames then
                ent = EntityLoad(effect[2])
            else
                ent = EntityLoad(constants.game_effects[effect[2]])
            end
            EntityAddChild(entity, ent)
            local_by_remote_id[effect_remote_id] = ent
            local com = EntityGetFirstComponentIncludingDisabled(ent, "GameEffectComponent")
            if com ~= nil then
                ComponentSetValue2(com, "frames", 999999999)
            end
        end
        ::continue::
        confirmed_effects[local_by_remote_id[effect_remote_id]] = true
    end

    local local_effects = effect_sync.get_ent_effects(entity)
    for _, effect in ipairs(local_effects) do
        if not confirmed_effects[effect] then
            EntityKill(effect)
        end
    end
end

return effect_sync