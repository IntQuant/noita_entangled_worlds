-- Called on clients when they get damage and redirects it to the host's hp over the network, ignoring any resists.
function damage_received(damage, message, entity_thats_responsible, is_fatal, projectile_thats_responsible)
    -- Change our health back
    local entity_id = GetUpdatedEntityID();
    local damageModelComponent = EntityGetFirstComponentIncludingDisabled( entity_id, "DamageModelComponent" )
    if damageModelComponent ~= nil then
        local health = ComponentGetValue2( damageModelComponent, "hp" )
        if health then
            ComponentSetValue2( damageModelComponent, "hp", health + damage )
        end
    end

    -- Damage the host
    CrossCall("ew_ds_damaged", damage, message)
end
