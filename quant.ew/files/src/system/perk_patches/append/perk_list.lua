local function patch_perk(perk_id, fn, ignore_original_func)
    local perk_data = get_perk_with_id(perk_list, perk_id)
    local old_func = perk_data.func
    perk_data.func = function(entity_perk_item, entity_who_picked, item_name, pickup_count)
        if not ignore_original_func then
            old_func(entity_perk_item, entity_who_picked, item_name, pickup_count)
        end
        fn(entity_perk_item, entity_who_picked, item_name, pickup_count)
    end
end

patch_perk("EXTRA_HP", function(entity_perk_item, entity_who_picked)
    if EntityHasTag(entity_who_picked, "player_unit") then
        CrossCall("ew_perks_modify_max_hp", 1.5, true)
    end
end)

patch_perk("VAMPIRISM", function(entity_perk_item, entity_who_picked)
    if EntityHasTag(entity_who_picked, "player_unit") then
        CrossCall("ew_perks_modify_max_hp", 0.75)
    end
end, true)