local inventory_helper = {}

local function entity_is_wand(entity_id)
	local ability_component = EntityGetFirstComponentIncludingDisabled(entity_id, "AbilityComponent")
    if ability_component == nil then return false end
	return ComponentGetValue2(ability_component, "use_gun_script") == true
end

function inventory_helper.get_inventory_items(player_data, inventory_name)
    local player = player_data.entity
    if(not player)then
        return {}
    end
    local inventory = nil 

    local player_child_entities = EntityGetAllChildren( player )
    if ( player_child_entities ~= nil ) then
        for i,child_entity in ipairs( player_child_entities ) do
            local child_entity_name = EntityGetName( child_entity )
            
            if ( child_entity_name == inventory_name ) then
                inventory = child_entity
            end
    
        end
    end

    if(inventory == nil)then
        return {}
    end

    local items = {}
    for i, v in ipairs(EntityGetAllChildren(inventory) or {}) do
        local item_component = EntityGetFirstComponentIncludingDisabled(v, "ItemComponent")
        if(item_component)then
            table.insert(items, v)
        end
    end
    return items
end

function inventory_helper.get_item_data(player_data, fresh)
    fresh = fresh or false

    local player = player_data.entity
    local inventory2Comp = EntityGetFirstComponentIncludingDisabled(player, "Inventory2Component")
    local mActiveItem = ComponentGetValue2(inventory2Comp, "mActiveItem")
    local wandData = {}
    local spellData = {}
    for k, item in ipairs(inventory_helper.get_inventory_items(player_data, "inventory_quick") or {}) do
        local item_comp = EntityGetFirstComponentIncludingDisabled(item, "ItemComponent")
        local slot_x, slot_y = ComponentGetValue2(item_comp, "inventory_slot")
        local item_x, item_y = EntityGetTransform(item)

        SetRandomSeed(item + slot_x + item_x, slot_y + item_y)

        -- local item_id = entity.GetVariable(item, "arena_entity_id")

        GlobalsSetValue(tostring(item) .. "_item", tostring(k))
        if(entity_is_wand(item))then
            local wand = EZWand(item)
            table.insert(wandData,
                {
                    data = wand:Serialize(not fresh, not fresh),
                    id = item_id or (item + Random(1, 10000000)),
                    slot_x = slot_x,
                    slot_y = slot_y,
                    active = (mActiveItem == item),
                    is_wand = true
                })
        else
            table.insert(wandData,
                {
                    data = np.SerializeEntity(item),
                    id = item_id or (item + Random(1, 10000000)),
                    slot_x = slot_x,
                    slot_y = slot_y,
                    active = (mActiveItem == item)
                })
        end
    end

    for k, item in ipairs(inventory_helper.get_inventory_items(player_data, "inventory_full") or {}) do
        local item_comp = EntityGetFirstComponentIncludingDisabled(item, "ItemComponent")
        local slot_x, slot_y = ComponentGetValue2(item_comp, "inventory_slot")
        local item_x, item_y = EntityGetTransform(item)

        SetRandomSeed(item + slot_x + item_x, slot_y + item_y)

        -- local item_id = entity.GetVariable(item, "arena_entity_id")

        GlobalsSetValue(tostring(item) .. "_item", tostring(k))
        if(entity_is_wand(item))then
            local wand = EZWand(item)
            table.insert(spellData,
                {
                    data = wand:Serialize(not fresh, not fresh),
                    -- id = item_id or (item + Random(1, 10000000)),
                    slot_x = slot_x,
                    slot_y = slot_y,
                    active = (mActiveItem == item),
                    is_wand = true
                })
        else
            table.insert(spellData,
                {
                    data = np.SerializeEntity(item),
                    -- id = item_id or (item + Random(1, 10000000)),
                    slot_x = slot_x,
                    slot_y = slot_y,
                    active = (mActiveItem == item)
                })
        end
    end

    return wandData, spellData
end

return inventory_helper