local entity_id = GetUpdatedEntityID()
local var
for _, v in ipairs(EntityGetComponent(entity_id, "VariableStorageComponent")) do
    if ComponentGetValue2(v, "name") == "ew_gid_lid" then
        var = v
    end
end
if var ~= nil and not ComponentGetValue2(var, "value_bool") then
    return
end

dofile_once("data/scripts/lib/utilities.lua")

local pos_x, pos_y = EntityGetTransform(entity_id)

local entity_pick_up_this_item = EntityLoad("data/entities/items/wand_level_03.xml", pos_x, pos_y)
local entity_ghost = entity_id
local itempickup = EntityGetFirstComponent(entity_ghost, "ItemPickUpperComponent")
if itempickup then
    ComponentSetValue2(itempickup, "only_pick_this_entity", entity_pick_up_this_item)
    GamePickUpInventoryItem(entity_ghost, entity_pick_up_this_item, false)
end

-- check that we hold the item
local items = GameGetAllInventoryItems(entity_ghost)
local has_item = false

if items ~= nil then
    for i, v in ipairs(items) do
        if v == entity_pick_up_this_item then
            has_item = true
        end
    end
end

-- if we don't have the item kill us for we are too dangerous to be left alive
if has_item == false then
    EntityKill(entity_ghost)
end
