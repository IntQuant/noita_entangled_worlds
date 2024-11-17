local function patch_perk_2(perk_id, fn)
    local perk_data = get_perk_with_id(perk_list, perk_id)
    local old_func = perk_data.func
    perk_data.func = function(entity_perk_item, entity_who_picked, item_name, pickup_count)
        fn(entity_perk_item, entity_who_picked, item_name, pickup_count, old_func)
    end
end

local function hide_perk(perk_id)
    --print("Hiding perk", perk_id)
    local perk_data = get_perk_with_id(perk_list, perk_id)
    perk_data.not_in_default_perk_pool = true
end

hide_perk("ABILITY_ACTIONS_MATERIALIZED")
hide_perk("HOMUNCULUS")

local ff = false
if CrossCall ~= nil then
    ff = CrossCall("ew_ff")
end
if ff then
    hide_perk("REVENGE_RATS")
end

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

local function string_split( s, splitter )
    local words = {};
    if s == nil or splitter == nil then
        return {}
    end
    for word in string.gmatch( s, '([^'..splitter..']+)') do
        table.insert( words, word );
    end
    return words;
end

--this crosscall check may break it but idc enough to test
local s = ""
if CrossCall ~= nil then
    s = CrossCall("ew_perk_ban_list")
end
for _, perk in ipairs(string_split(s, ',')) do
    hide_perk(perk)
end