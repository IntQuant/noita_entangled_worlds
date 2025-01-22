function death(damage_type_bit_field, damage_message, entity_thats_responsible, drop_items)
    local ent = GetUpdatedEntityID()
    CrossCall("ew_death_notify", ent, EntityGetFilename(ent), entity_thats_responsible)
end
