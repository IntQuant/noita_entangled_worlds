local old_EntityLoad = EntityLoad

function EntityLoad(ent, x, y)
    local pre_result = CrossCall("ew_spawn_hook_pre", ent, x, y)
    -- pre_result can be not a bool in case it spawned the entity itself.
    if pre_result == true then
        local ent_id = old_EntityLoad(ent, x, y)
        CrossCall("ew_spawn_hook_post", ent, ent_id)
        return ent_id
    end
    if pre_result ~= false then
        return pre_result
    end
end
