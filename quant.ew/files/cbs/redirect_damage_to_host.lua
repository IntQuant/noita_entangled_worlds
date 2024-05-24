-- Called on host when clients get damage and redirects it straight to the host's hp, ignoring any resists.
function damage_received(damage, message, entity_thats_responsible, is_fatal, projectile_thats_responsible)
    -- Damage the host
    local host_entity_id = EntityGetWithTag("ew_host")[1]
	local protection_component_id = GameGetGameEffect(host_entity_id, "PROTECTION_ALL")
	if protection_component_id then
        EntitySetComponentIsEnabled(host_entity_id, protection_component_id, false)
	end
    EntityInflictDamage(host_entity_id, damage, "DAMAGE_CURSE", message, "NONE", 0, 0, entity_thats_responsible)
    if protection_component_id then
        EntitySetComponentIsEnabled(host_entity_id, protection_component_id, true)
    end
end
