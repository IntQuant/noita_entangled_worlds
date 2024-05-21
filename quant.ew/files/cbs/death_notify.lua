function death( damage_type_bit_field, damage_message, entity_thats_responsible, drop_items )
    local i = 1
    while GlobalsGetValue("ew_enemy_death_"..i, "0") ~= "0" do
        i = i + 1
    end
    GlobalsSetValue("ew_enemy_death_"..i, tostring(GetUpdatedEntityID()))
end