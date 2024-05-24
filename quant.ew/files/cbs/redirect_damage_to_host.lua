function inflict_direct_damage(entity_id, damage)
    local damageModelComponent = EntityGetFirstComponentIncludingDisabled(entity_id, "DamageModelComponent")
    if not damageModelComponent then
        return
    end
    local health = ComponentGetValue2(damageModelComponent, "hp")
    if not health then
        return
    end
    new_health = health - damage
    ComponentSetValue2(damageModelComponent, "hp", new_health)
    if new_health <= 0 then
        EntityKill(entity_id)
    end
    
end

-- Called on host when clients get damage and redirects it straight to the host's hp, ignoring any resists.
function damage_received(damage, message, entity_thats_responsible, is_fatal, projectile_thats_responsible)
    -- Damage the host
    local host_entity_id = EntityGetWithTag("ew_host")[1]
    inflict_direct_damage(host_entity_id, damage)
    -- Return the health to the client
    local client_entity_id = GetUpdatedEntityID(client_entity_id, -damage)
    inflict_direct_damage(client_entity_id, -damage)
end
