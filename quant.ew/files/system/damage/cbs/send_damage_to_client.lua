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
    local gid
    for _, v in
        ipairs(
            EntityGetComponent(
                (projectile_thats_responsible ~= 0 and projectile_thats_responsible) or entity_thats_responsible,
                "VariableStorageComponent"
            ) or {}
        )
    do
        if ComponentGetValue2(v, "name") == "ew_gid_lid" then
            gid = v
            break
        end
    end
    if dtypes == 1 and gid ~= nil and ComponentGetValue2(gid, "value_bool") then
        -- Damage the client
        CrossCall("ew_ds_client_damaged", EntityGetName(entity_id), damage, message)
    end
end
