function damage_about_to_be_received( damage, x, y, entity_thats_responsible, critical_hit_chance )
    local dtypes = GetDamageDetails().damage_types
    if GameHasFlagRun("ew_flag_this_is_host") and dtypes == 1
            and entity_thats_responsible ~= GameGetWorldStateEntity() then
        return damage, 0
    end

    if entity_thats_responsible ~= GameGetWorldStateEntity() then
        return 2^-128, 0
    end

    if damage < 0 then
        return -2^-128, 0
    end

    return damage, 0
 end