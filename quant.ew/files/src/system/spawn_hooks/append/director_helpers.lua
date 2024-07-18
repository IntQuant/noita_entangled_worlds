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