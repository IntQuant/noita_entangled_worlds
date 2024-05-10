function damage_received( damage, message, entity_thats_responsible, is_fatal, projectile_thats_responsible )
    local entity_id = GetUpdatedEntityID()
    local damageModelComponent = EntityGetFirstComponentIncludingDisabled( entity_id, "DamageModelComponent" )
    if damageModelComponent ~= nil then
        local health = ComponentGetValue2( damageModelComponent, "hp" )
        if health then
            ComponentSetValue2( damageModelComponent, "hp", health + damage )
        end
    end
    local host_ent = EntityGetWithTag("ew_host")[1]
    if host_ent ~= nil then        
        EntityInflictDamage(host_ent, damage, "DAMAGE_CURSE", message, "NONE", 0, 0, entity_thats_responsible)
    end
end