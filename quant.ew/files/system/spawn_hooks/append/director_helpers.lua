local old_EntityLoadCameraBound = EntityLoadCameraBound

function EntityLoadCameraBound(ent, x, y)
    local pre_result = CrossCall("ew_spawn_hook_pre", ent, x, y)
    if pre_result == true then
        local ent_id = old_EntityLoadCameraBound(ent, x, y)
        CrossCall("ew_spawn_hook_post", ent, ent_id)
        return ent_id
    end
    if pre_result ~= false then
        return pre_result
    end
end

local old_CreateItemActionEntity = CreateItemActionEntity

function CreateItemActionEntity(action_id, x, y)
    if not CrossCall("ew_action_spawn_hook_pre") then
        return 0
    end
    local eid = old_CreateItemActionEntity(action_id, x, y)
    CrossCall("ew_action_spawn_hook", eid)
    return eid
end
