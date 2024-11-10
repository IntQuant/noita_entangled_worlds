local np = require("noitapatcher")
local EZWand = dofile_once("mods/quant.ew/files/lib/EZWand.lua")
local util = dofile_once("mods/quant.ew/files/core/util.lua")
local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")

local inventory_helper = {}

local function entity_is_wand(entity_id)
    local ability_component = EntityGetFirstComponentIncludingDisabled(entity_id, "AbilityComponent")
    if ability_component == nil then return false end
    return ComponentGetValue2(ability_component, "use_gun_script") == true
end

function inventory_helper.get_all_inventory_items(player_data)
    local items = GameGetAllInventoryItems(player_data.entity) or {}
    local result = {}
    local kill = {}
    for _, item in pairs(items) do
        if EntityGetFilename(item) == "data/entities/base_item.xml" then
            table.insert(kill, item)
            goto continue
        end
        table.insert(result, item)
        for _, sub_item in ipairs(EntityGetAllChildren(item) or {}) do
            table.insert(result, sub_item)
        end
        ::continue::
    end
    for _, item in ipairs(kill) do
        EntityKill(item)
    end
    return result
end

function inventory_helper.get_inventory_items(player_data, inventory_name)
    local player = player_data.entity
    if(not player)then
        return {}
    end
    local inventory

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

local ability_component_extra_fields = {"stat_times_player_has_shot", "stat_times_player_has_edited", "gun_level"}

function inventory_helper.serialize_single_item(item)
    local item_data
    local x, y = EntityGetTransform(item)
    if(entity_is_wand(item))then
        local wand = EZWand(item)
        local extra = {}
        local ability = EntityGetFirstComponentIncludingDisabled(item, "AbilityComponent")
        if ability and ability ~= 0 then
            for i, field in ipairs(ability_component_extra_fields) do
                extra[i] = ComponentGetValue2(ability, field)
            end
        end
        local is_new = true
        local item_component = EntityGetFirstComponentIncludingDisabled(item, "ItemComponent")
        local image_inv
        if item_component and item_component ~= 0 then
            image_inv = ComponentGetValue2(item_component, "ui_sprite")
            is_new = ComponentGetValue2(item_component, "play_hover_animation")
        end
        local vx = 0
        local vy = 0
        local vel = EntityGetFirstComponentIncludingDisabled(item, "VelocityComponent")
        if vel and vel ~= 0 then
            vx, vy = ComponentGetValue2(vel, "mVelocity")
        end
        local sprite = EntityGetFirstComponentIncludingDisabled(item, "SpriteComponent")
        if sprite ~= nil then
            sprite = ComponentGetValue2(sprite, "image_file")
        end
        local varp = EntityGetFilename(item) == "data/entities/items/wand_varpuluuta.xml"
        GamePrint(tostring(image_inv))
        item_data = {true, wand:Serialize(true, true), x, y, extra, is_new, {vx, vy}, sprite, image_inv, varp}
    else
        item_data = {false, util.serialize_entity(item), x, y}
    end
    local item_cost_component = EntityGetFirstComponentIncludingDisabled(item, "ItemCostComponent")
    if item_cost_component and item_cost_component ~= 0 then
        local cost = ComponentGetValue2(item_cost_component, "cost")
        local stealable = ComponentGetValue2(item_cost_component, "stealable")
        item_data.shop_info = {cost, stealable}
    end

    return item_data
end

function inventory_helper.deserialize_single_item(item_data)
    local item
    local x, y = item_data[3], item_data[4]
    if item_data[1] then
        item = EZWand(item_data[2], x, y, false).entity_id
        local extra = item_data[5]
        local is_new = item_data[6]
        local vx, vy = item_data[7][1], item_data[7][2]
        local image = item_data[8]
        local image_inv = item_data[9]
        local sprite = EntityGetFirstComponentIncludingDisabled(item, "SpriteComponent")
        if sprite ~= nil then
            ComponentSetValue2(sprite, "image_file", image)
        end
        if item_data[10] then
            local varp = EntityCreateNew()
            EntityAddComponent2(varp, "InheritTransformComponent", {_tags="enabled_in_world,enabled_in_hand", only_position=true, parent_hotspot_tag="shoot_pos"})
            EntityAddComponent2(varp, "CellEaterComponent", {_tags="enabled_in_world,enabled_in_hand", radius=20, eat_probability=10, only_stain=true})
            EntityAddChild(item, varp)
        end
        local ability = EntityGetFirstComponentIncludingDisabled(item, "AbilityComponent")
        local item_component = EntityGetFirstComponentIncludingDisabled(item, "ItemComponent")
        if item_component ~= nil then
            ComponentSetValue2(item_component, "ui_sprite", image_inv)
        end
        if ability ~= nil and extra ~= nil then
            for i, field in ipairs(ability_component_extra_fields) do
                if extra[i] ~= nil then
                    ComponentSetValue2(ability, field, extra[i])
                end
            end
        end
        if not is_new then
            ComponentSetValue2(item_component, "play_hover_animation", false)
            local phys = EntityGetFirstComponentIncludingDisabled(item, "SimplePhysicsComponent")
            EntitySetComponentIsEnabled(item, phys, true)
            local part = EntityGetFirstComponentIncludingDisabled(item, "SpriteParticleEmitterComponent")
            EntitySetComponentIsEnabled(item, part, false)
            local vel = EntityGetFirstComponentIncludingDisabled(item, "VelocityComponent")
            if vel and vel ~= 0 then
                ComponentSetValue2(vel, "mVelocity", vx, vy)
            end
        end
        -- EntityAddTag(item, "does_physics_update")
    else
        item = util.deserialize_entity(item_data[2], x, y)
    end

    if item_data.shop_info ~= nil then
        local item_cost_component = util.get_or_create_component(item, "ItemCostComponent")
        ComponentAddTag(item_cost_component, "enabled_in_world")
        ComponentAddTag(item_cost_component, "shop_cost")
        ComponentSetValue2(item_cost_component, "cost", item_data.shop_info[1])
        if item_data.gid == nil then
            ComponentSetValue2(item_cost_component, "stealable", false)
            print("ERROR: why is ".. tostring(item) .. " gid nil")
        elseif string.sub(item_data.gid, 1, 16) ~= ctx.my_id then
            ComponentSetValue2(item_cost_component, "stealable", false)
        else
            local mx, my = GameGetCameraPos()
            if (math.abs(mx - x) > 1024 or math.abs(my - y) > 1024) and ComponentGetValue2(item_cost_component, "stealable") then
                EntityAddComponent2(item, "VariableStorageComponent", {_tags = "ew_try_stealable"})
                ComponentSetValue2(item_cost_component, "stealable", false)
            end
        end

        util.ensure_component_present(item, "SpriteComponent", "shop_cost", {
            image_file = "data/fonts/font_pixel_white.xml",
            is_text_sprite = true,
            offset_x = 7,
            offset_y = 25,
            alpha = 1,
            z_index = -1,
            update_transform_rotation = false,
        }, "shop_cost,enabled_in_world")
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
    for _, item in ipairs(inventory_helper.get_inventory_items(player_data, "inventory_quick") or {}) do
        local item_comp = EntityGetFirstComponentIncludingDisabled(item, "ItemComponent")
        local slot_x, slot_y = ComponentGetValue2(item_comp, "inventory_slot")
        local item_x, item_y = EntityGetTransform(item)

        SetRandomSeed(item + slot_x + item_x, slot_y + item_y)

        if(entity_is_wand(item))then
            table.insert(wandData,
                {
                    data = inventory_helper.serialize_single_item(item),
                    slot_x = slot_x,
                    slot_y = slot_y,
                    active = (mActiveItem == item),
                    is_wand = true,
                    old_id = item
                })
        else
            table.insert(wandData,
                {
                    data = inventory_helper.serialize_single_item(item),
                    slot_x = slot_x,
                    slot_y = slot_y,
                    active = (mActiveItem == item)
                })
        end
    end

    for _, item in ipairs(inventory_helper.get_inventory_items(player_data, "inventory_full") or {}) do
        local item_comp = EntityGetFirstComponentIncludingDisabled(item, "ItemComponent")
        local slot_x, slot_y = ComponentGetValue2(item_comp, "inventory_slot")
        local item_x, item_y = EntityGetTransform(item)

        SetRandomSeed(item + slot_x + item_x, slot_y + item_y)

        -- local item_id = entity.GetVariable(item, "arena_entity_id")

        -- GlobalsSetValue(tostring(item) .. "_item", tostring(k))
        if(entity_is_wand(item))then
            table.insert(spellData,
                {
                    data = inventory_helper.serialize_single_item(item),
                    -- id = item_id or (item + Random(1, 10000000)),
                    slot_x = slot_x,
                    slot_y = slot_y,
                    active = (mActiveItem == item),
                    is_wand = true
                })
        else
            table.insert(spellData,
                {
                    data = inventory_helper.serialize_single_item(item),
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
    local entity_children = EntityGetAllChildren(entity)
    if entity_children ~= nil then
        for _, child in pairs( entity_children ) do
            if EntityGetName( child ) == "inventory_quick" then
                EntityAddChild( child, item)
            end
        end
    end

    EntitySetComponentsWithTagEnabled( item, "enabled_in_world", false )
    EntitySetComponentsWithTagEnabled( item, "enabled_in_hand", false )
    EntitySetComponentsWithTagEnabled( item, "enabled_in_inventory", true )

    local wand_children = EntityGetAllChildren(item) or {}

    for _, _ in ipairs(wand_children)do
      EntitySetComponentsWithTagEnabled( item, "enabled_in_world", false )
    end
end

local function remove_non_send(item)
    for _, com in ipairs(EntityGetAllComponents(item) or {}) do
       if ComponentHasTag(com, "ew_remove_on_send") then
        EntityRemoveComponent(item, com)
       end
    end
end

function inventory_helper.set_item_data(item_data, player_data)
    local player = player_data.entity
    if player == nil or not EntityGetIsAlive(player) then
        if player ~= nil then
            GamePrint("Skip set_item_data, player ".. player_data.name .. " " .. player_data.entity .. " is dead")
        end
        return
    end

    local children = EntityGetAllChildren(player) or {}
    for _, child in pairs(children) do
        if EntityGetName(child) == "inventory_quick" then
            local inv = EntityGetAllChildren(child)
            if inv ~= nil then
                for _, item in pairs(inv) do
                    EntityKill(item)
                end
            end
        end
    end


    if (item_data ~= nil) then
        local active_item_entity

        for _, itemInfo in ipairs(item_data) do
            --local x, y = EntityGetTransform(player)
            local item_entity
            local item
            if(itemInfo.is_wand)then
                item = inventory_helper.deserialize_single_item(itemInfo.data)
                remove_non_send(item)
                item = EZWand(item)
            else
                item = inventory_helper.deserialize_single_item(itemInfo.data)
                remove_non_send(item)
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
    async(function()
        wait(1)
        local inventory2Comp = EntityGetFirstComponentIncludingDisabled(player, "Inventory2Component")
        if inventory2Comp ~= nil then
            ComponentSetValue2(inventory2Comp, "mForceRefresh", true)
        end
    end)
end

function inventory_helper.has_inventory_changed(player_data)
    local prev_inventory = player_data.prev_inventory_hash
    local inventory_hash = 0
    if player_data.entity == nil or not EntityGetIsAlive(player_data.entity) or GameGetAllInventoryItems(player_data.entity) == nil then
        return false
    end
    for _, item in ipairs(GameGetAllInventoryItems(player_data.entity) or {}) do
        local item_comp = EntityGetFirstComponentIncludingDisabled(item, "ItemComponent")
        local slot_x, slot_y = ComponentGetValue2(item_comp, "inventory_slot")
        inventory_hash = (inventory_hash*19 + (item % 65000 + slot_x + slot_y)) % (math.pow(2, 20) - 1)
    end
    player_data.prev_inventory_hash = inventory_hash
    return inventory_hash ~= prev_inventory
end

return inventory_helper