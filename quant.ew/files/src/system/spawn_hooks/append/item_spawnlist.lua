local old_EntityLoad = EntityLoad

function EntityLoad(ent, x, y)
    if CrossCall("ew_spawn_hook_pre", ent, x, y) then
        local ent_id = old_EntityLoad(ent, x, y)
        CrossCall("ew_spawn_hook_post", ent, ent_id)
    end
end