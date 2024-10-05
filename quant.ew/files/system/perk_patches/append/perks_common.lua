local function patch_perk_2(perk_id, fn)
    local perk_data = get_perk_with_id(perk_list, perk_id)
    local old_func = perk_data.func
    perk_data.func = function(entity_perk_item, entity_who_picked, item_name, pickup_count)
        fn(entity_perk_item, entity_who_picked, item_name, pickup_count, old_func)
    end
end

local function hide_perk(perk_id)
    local perk_data = get_perk_with_id(perk_list, perk_id)
    perk_data.not_in_default_perk_pool = true
end

hide_perk("ABILITY_ACTIONS_MATERIALIZED")
hide_perk("TELEKINESIS")
hide_perk("CORDYCEPS")
hide_perk("HOMUNCULUS")
hide_perk("REVENGE_EXPLOSION")
hide_perk("EXTRA_SHOP_ITEM")

patch_perk_2("SHIELD", function(entity_perk_item, entity_who_picked, item_name, pickup_count, orig_fn)
    GlobalsSetValue("PERK_SHIELD_COUNT", tostring(pickup_count-1))
    orig_fn(entity_perk_item, entity_who_picked, item_name, pickup_count)
end)

patch_perk_2("ATTACK_FOOT", function(entity_perk_item, entity_who_picked, item_name, pickup_count, orig_fn)
    if pickup_count ~= 1 then
        GameHasFlagRun("ATTACK_FOOT_CLIMBER")
    else
        GameRemoveFlagRun("ATTACK_FOOT_CLIMBER")
    end
    orig_fn(entity_perk_item, entity_who_picked, item_name, pickup_count)
end)

patch_perk_2("LEGGY_FEET", function(entity_perk_item, entity_who_picked, item_name, pickup_count, orig_fn)
    if pickup_count ~= 1 then
        GameHasFlagRun("ATTACK_FOOT_CLIMBER")
    else
        GameRemoveFlagRun("ATTACK_FOOT_CLIMBER")
    end
    orig_fn(entity_perk_item, entity_who_picked, item_name, pickup_count)
end)