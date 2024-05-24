-- Called on host when clients get damage and redirects it straight to the host's hp, ignoring any resists.
function damage_received(damage, message, entity_thats_responsible, is_fatal, projectile_thats_responsible)
    local host_entity_id = EntityGetWithTag("ew_host")[1]
    local host_damageModelComponent = EntityGetFirstComponentIncludingDisabled(host_entity_id, "DamageModelComponent")
    if not host_damageModelComponent then
        return
    end
    local health = ComponentGetValue2(host_damageModelComponent, "hp")
    if not health then
        return
    end
    ComponentSetValue2(host_damageModelComponent, "hp", health - damage)

    -- Change our health back
    local entity_id = GetUpdatedEntityID();
    local damageModelComponent = EntityGetFirstComponentIncludingDisabled( entity_id, "DamageModelComponent" )
    if damageModelComponent ~= nil then
        local health = ComponentGetValue2( damageModelComponent, "hp" )
        if health then
            ComponentSetValue2( damageModelComponent, "hp", health + damage )
        end
    end
end
