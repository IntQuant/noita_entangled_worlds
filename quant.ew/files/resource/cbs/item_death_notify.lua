function death(damage_type_bit_field, damage_message, entity_thats_responsible, drop_items)
    CrossCall("ew_item_death_notify", GetUpdatedEntityID(), entity_thats_responsible)
end
