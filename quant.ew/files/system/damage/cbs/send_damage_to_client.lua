function damage_received(damage, message, entity_thats_responsible, is_fatal, projectile_thats_responsible)
    if
        entity_thats_responsible == GameGetWorldStateEntity()
        or (projectile_thats_responsible ~= nil and projectile_thats_responsible ~= 0)
        or EntityHasTag(entity_thats_responsible, "ew_peer")
    then
        return
    end

    -- Change our health back
    local entity_id = GetUpdatedEntityID()

    local dtypes = GetDamageDetails().damage_types
    -- Only handle melee damage that way.
    if dtypes == 1 and CrossCall("ew_do_i_own", entity_thats_responsible) then
        -- Damage the client
        CrossCall("ew_ds_client_damaged", EntityGetName(entity_id), damage, message)
    end
end
