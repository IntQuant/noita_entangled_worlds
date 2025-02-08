function death(damage_type_bit_field, damage_message, entity_thats_responsible, drop_items)
    local ent = GetUpdatedEntityID()
    local x, y = EntityGetTransform(ent)
    local wait_on_kill = false
    local damage = EntityGetFirstComponentIncludingDisabled(ent, "DamageModelComponent")
    if damage ~= nil then
        wait_on_kill = ComponentGetValue2(damage, "wait_for_kill_flag_on_death")
    end
    CrossCall("ew_death_notify", ent, wait_on_kill, x, y, EntityGetFilename(ent), entity_thats_responsible)
end
