local wandfinder = {}

local function entity_is_wand(entity_id)
	local ability_component = EntityGetFirstComponentIncludingDisabled(entity_id, "AbilityComponent")
    if ability_component == nil then return false end
	return ComponentGetValue2(ability_component, "use_gun_script") == true
end

local function get_all_wands()
    local wands = {}
    local items = GameGetAllInventoryItems(ctx.my_player.entity) or {}
    for _, item in ipairs(items) do
        if entity_is_wand(item) then
            table.insert(wands, item)
        end
    end
    return wands
end

-- Just return the first wand for now.
function wandfinder.find_attack_wand()
    local wands = get_all_wands()
    if #wands == 0 then
        return nil
    end
    local best_wand = wands[1]
    local best_slot = 1000
    for _, wand in ipairs(wands) do
        local item_c = EntityGetFirstComponentIncludingDisabled(wand, "ItemComponent")
        local slot_x = ComponentGetValue2(item_c, "inventory_slot")
        if slot_x < best_slot then
            best_wand = wand
            best_slot = slot_x
        end
    end
    return best_wand
end

return wandfinder