local old_EntityLoadCameraBound = EntityLoadCameraBound

function EntityLoadCameraBound(ent, x, y)
    if CrossCall("ew_spawn_hook_pre", ent, x, y) then
        local ent_id = old_EntityLoadCameraBound(ent, x, y)
        CrossCall("ew_spawn_hook_post", ent, ent_id)
    end
end