function damage_about_to_be_received(damage, x, y, entity_thats_responsible, critical_hit_chance)
    local dtypes = GetDamageDetails().damage_types
    if dtypes == 524288 or dtypes == 8192 then
        return damage, 0
    elseif damage == 0 then
        return 0, 0
    elseif damage > 0 then
        return 2 ^ -38, 0
    else
        return -2 ^ -38, 0
    end
end
