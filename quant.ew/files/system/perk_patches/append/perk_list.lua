local function patch_perk(perk_id, ignore_original_func, fn)
    local perk_data = get_perk_with_id(perk_list, perk_id)
    local old_func = perk_data.func
    perk_data.func = function(entity_perk_item, entity_who_picked, item_name, pickup_count)
        if not ignore_original_func then
            old_func(entity_perk_item, entity_who_picked, item_name, pickup_count)
        end
        fn(entity_perk_item, entity_who_picked, item_name, pickup_count)
    end
end

local function hide_perk(perk_id)
    local perk_data = get_perk_with_id(perk_list, perk_id)
    perk_data.not_in_default_perk_pool = true
end

patch_perk("EXTRA_HP", false, function(entity_perk_item, entity_who_picked)
    if EntityHasTag(entity_who_picked, "player_unit") then
        CrossCall("ew_perks_modify_max_hp", 1.5, true)
    end
end)

patch_perk("VAMPIRISM", true, function(entity_perk_item, entity_who_picked)
    if EntityHasTag(entity_who_picked, "player_unit") then
        CrossCall("ew_perks_modify_max_hp", 0.75)
    end
end)

patch_perk("GLASS_CANNON", true, function()
    -- Intentionally empty.
end)

hide_perk("ABILITY_ACTIONS_MATERIALIZED")
hide_perk("RESPAWN")
hide_perk("TELEKINESIS")
hide_perk("SAVING_GRACE")
hide_perk("INVISIBILITY")
hide_perk("CORDYCEPS")
hide_perk("HOMUNCULUS")
