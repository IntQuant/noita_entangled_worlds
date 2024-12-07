-- Called on clients when they get damage and redirects it to the host's hp over the network, ignoring any resists.
function damage_received(damage, message, entity_thats_responsible, is_fatal, projectile_thats_responsible)
    -- Damage the host
    CrossCall("ew_damage_message", message, entity_thats_responsible)
end