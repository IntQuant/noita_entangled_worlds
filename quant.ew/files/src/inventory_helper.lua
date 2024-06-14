local np = require("noitapatcher")
local EZWand = dofile_once("mods/quant.ew/files/lib/EZWand.lua")
local pretty = dofile_once("mods/quant.ew/files/lib/pretty_print.lua")

local inventory_helper = {}

local function entity_is_wand(entity_id)
	local ability_component = EntityGetFirstComponentIncludingDisabled(entity_id, "AbilityComponent")
    if ability_component == nil then return false end
	return ComponentGetValue2(ability_component, "use_gun_script") == true
end

function inventory_helper.get_all_inventory_items(player_data)
    local items = GameGetAllInventoryItems(player_data.entity) or {}
    local result = {}
    for _, item in pairs(items) do
        table.insert(result, item)
        for _, sub_item in pairs(EntityGetAllChildren(item) or {}) do
            table.insert(result, sub_item)
        end
    end
    return result
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

function inventory_helper.serialize_single_item(item)
    local x, y = EntityGetTransform(item)
    if(entity_is_wand(item))then
        local wand = EZWand(item)
        return {true, wand:Serialize(true, true), x, y}
    else
        return {false, np.SerializeEntity(item), x, y}
    end
end

function inventory_helper.deserialize_single_item(item_data)
    local item = nil
    local x, y = item_data[3], item_data[4]
    if item_data[1] then
        item = EZWand(item_data[2], x, y, false).entity_id
        -- EntityAddTag(item, "does_physics_update")
    else
        item = EntityCreateNew()
        np.DeserializeEntity(item, item_data[2], x, y)
    end
    return item
end

function inventory_helper.get_item_data(player_data, fresh)
    fresh = fresh or false

    local player = player_data.entity
    local inventory2Comp = EntityGetFirstComponentIncludingDisabled(player, "Inventory2Component")
    if (not inventory2Comp) or inventory2Comp == 0 then
        return {}, {}
    end
    
    local mActiveItem = ComponentGetValue2(inventory2Comp, "mActiveItem")
    local wandData = {}
    local spellData = {}
    for k, item in ipairs(inventory_helper.get_inventory_items(player_data, "inventory_quick") or {}) do
        local item_comp = EntityGetFirstComponentIncludingDisabled(item, "ItemComponent")
        local slot_x, slot_y = ComponentGetValue2(item_comp, "inventory_slot")
        local item_x, item_y = EntityGetTransform(item)

        SetRandomSeed(item + slot_x + item_x, slot_y + item_y)
        -- GlobalsSetValue(tostring(item) .. "_item", tostring(k))
        if(entity_is_wand(item))then
            local wand = EZWand(item)
            table.insert(wandData,
                {
                    data = wand:Serialize(not fresh, not fresh),
                    -- id = item_id or (item + Random(1, 10000000)),
                    slot_x = slot_x,
                    slot_y = slot_y,
                    active = (mActiveItem == item),
                    is_wand = true
                })
        else
            table.insert(wandData,
                {
                    data = np.SerializeEntity(item),
                    -- id = item_id or (item + Random(1, 10000000)),
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

        -- GlobalsSetValue(tostring(item) .. "_item", tostring(k))
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

local function pickup_item(entity, item)
    local item_component = EntityGetFirstComponentIncludingDisabled(item, "ItemComponent")
    if item_component then
      ComponentSetValue2(item_component, "has_been_picked_by_player", true)
    end
    local entity_children = EntityGetAllChildren(entity) or {}
    for key, child in pairs( entity_children ) do
      if EntityGetName( child ) == "inventory_quick" then
        EntityAddChild( child, item)
      end
    end
  
    EntitySetComponentsWithTagEnabled( item, "enabled_in_world", false )
    EntitySetComponentsWithTagEnabled( item, "enabled_in_hand", false )
    EntitySetComponentsWithTagEnabled( item, "enabled_in_inventory", true )
  
    local wand_children = EntityGetAllChildren(item) or {}
  
    for k, v in ipairs(wand_children)do
      EntitySetComponentsWithTagEnabled( item, "enabled_in_world", false )
    end  
end

function inventory_helper.set_item_data(item_data, player_data)
    local player = player_data.entity
    if (not EntityGetIsAlive(player)) then
        GamePrint("Skip set_item_data, player ".. player_data.name .. " " .. player_data.entity .. " is dead")
        return
    end

    local items = GameGetAllInventoryItems(player) or {}
    for i, item_id in ipairs(items) do
        GameKillInventoryItem(player, item_id)
        EntityKill(item_id)
    end


    if (item_data ~= nil) then
        local active_item_entity = nil

        for k, itemInfo in ipairs(item_data) do
            local x, y = EntityGetTransform(player)
            local item_entity = nil
            local item = nil
            if(itemInfo.is_wand)then
                item = EZWand(itemInfo.data, x, y, GameHasFlagRun("refresh_all_charges"))
                
            else
                item = EntityCreateNew()
                np.DeserializeEntity(item, itemInfo.data, x, y)
            end

            if (item == nil) then
                return
            end

            if(itemInfo.is_wand)then
                EntityAddTag(item.entity_id, "ew_client_item")
                item:PickUp(player)
                local itemComp = EntityGetFirstComponentIncludingDisabled(item.entity_id, "ItemComponent")
                if (itemComp ~= nil) then
                    ComponentSetValue2(itemComp, "inventory_slot", itemInfo.slot_x, itemInfo.slot_y)
                end
                item_entity = item.entity_id
                if (itemInfo.active) then
                    active_item_entity = item.entity_id
                end
            else
                EntityAddTag(item, "ew_client_item")
                pickup_item(player, item)
                local itemComp = EntityGetFirstComponentIncludingDisabled(item, "ItemComponent")
                if (itemComp ~= nil) then
                    ComponentSetValue2(itemComp, "inventory_slot", itemInfo.slot_x, itemInfo.slot_y)
                end
                item_entity = item
                if (itemInfo.active) then
                    active_item_entity = item
                end
            end

            --print("Deserialized wand #"..tostring(k).." - Active? "..tostring(wandInfo.active))

            -- entity.SetVariable(item_entity, "arena_entity_id", itemInfo.id)

            -- local lua_comps = EntityGetComponentIncludingDisabled(item_entity, "LuaComponent") or {}
            -- local has_pickup_script = false
            -- for i, lua_comp in ipairs(lua_comps) do
            --     if (ComponentGetValue2(lua_comp, "script_item_picked_up") == "mods/evaisa.arena/files/scripts/gamemode/misc/item_pickup.lua") then
            --         has_pickup_script = true
            --     end
            -- end

            -- if (not has_pickup_script) then
            --     EntityAddTag(item_entity, "does_physics_update")
            --     EntityAddComponent(item_entity, "LuaComponent", {
            --         _tags = "enabled_in_world,enabled_in_hand,enabled_in_inventory",
            --         -- script_item_picked_up = "mods/evaisa.arena/files/scripts/gamemode/misc/item_pickup.lua",
            --         -- script_kick = "mods/evaisa.arena/files/scripts/gamemode/misc/item_kick.lua",
            --         -- script_throw_item = "mods/evaisa.arena/files/scripts/gamemode/misc/item_throw.lua",
            --     })
            -- end
        end

        if (active_item_entity ~= nil) then
            np.SetActiveHeldEntity(player, active_item_entity, false, false)
        end
    end
end

function inventory_helper.has_inventory_changed(player_data)
    local prev_inventory = player_data.prev_inventory_hash
    
    local inventory_hash = 0
    for _, item in ipairs(GameGetAllInventoryItems(player_data.entity) or {}) do
        local item_comp = EntityGetFirstComponentIncludingDisabled(item, "ItemComponent")
        local slot_x, slot_y = ComponentGetValue2(item_comp, "inventory_slot")
        inventory_hash = (inventory_hash*19 + (item % 65000 + slot_x + slot_y)) % (math.pow(2, 20) - 1)
    end
    player_data.prev_inventory_hash = inventory_hash
    return inventory_hash ~= prev_inventory
end

return inventory_helper

