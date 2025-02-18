local EZWand = dofile_once("mods/quant.ew/files/lib/EZWand.lua")

local inventory_helper = {}

local function entity_is_wand(entity_id)
    local ability_component = EntityGetFirstComponentIncludingDisabled(entity_id, "AbilityComponent")
    if ability_component == nil then
        return false
    end
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
    if not player then
        return {}
    end
    local inventory

    local player_child_entities = EntityGetAllChildren(player)
    if player_child_entities ~= nil then
        for i, child_entity in ipairs(player_child_entities) do
            local child_entity_name = EntityGetName(child_entity)

            if child_entity_name == inventory_name then
                inventory = child_entity
            end
        end
    end

    if inventory == nil then
        return {}
    end

    local items = {}
    for i, v in ipairs(EntityGetAllChildren(inventory) or {}) do
        local item_component = EntityGetFirstComponentIncludingDisabled(v, "ItemComponent")
        if item_component then
            table.insert(items, v)
        end
    end
    return items
end

function inventory_helper.serialize_single_item(item)
    local x, y = EntityGetTransform(item)
    local item_data = { util.serialize_entity(item), x, y }
    local item_cost_component = EntityGetFirstComponentIncludingDisabled(item, "ItemCostComponent")
    if item_cost_component and item_cost_component ~= 0 then
        local cost = ComponentGetValue2(item_cost_component, "cost")
        local stealable = ComponentGetValue2(item_cost_component, "stealable")
        item_data.shop_info = { cost, stealable }
    end

    return item_data
end

function inventory_helper.deserialize_single_item(item_data)
    local x, y = item_data[2], item_data[3]
    local item = util.deserialize_entity(item_data[1], x, y)
    local ability = EntityGetFirstComponentIncludingDisabled(item, "AbilityComponent")
    if ability ~= nil then
        ComponentSetValue2(ability, "mNextFrameUsable", 0)
        ComponentSetValue2(ability, "mCastDelayStartFrame", 0)
        ComponentSetValue2(ability, "mReloadNextFrameUsable", 0)
    end

    if item_data.shop_info ~= nil then
        local item_cost_component = util.get_or_create_component(item, "ItemCostComponent")
        ComponentAddTag(item_cost_component, "enabled_in_world")
        ComponentAddTag(item_cost_component, "shop_cost")
        ComponentSetValue2(item_cost_component, "cost", item_data.shop_info[1])
        if item_data.gid == nil then
            ComponentSetValue2(item_cost_component, "stealable", false)
            print("ERROR: why is " .. tostring(item) .. " gid nil")
        elseif string.sub(item_data.gid, 1, 16) ~= ctx.my_id then
            ComponentSetValue2(item_cost_component, "stealable", false)
        else
            local mx, my = GameGetCameraPos()
            if math.abs(mx - x) > 1024 or math.abs(my - y) > 1024 then
                if ComponentGetValue2(item_cost_component, "stealable") then
                    EntityAddComponent2(item, "VariableStorageComponent", { _tags = "ew_try_stealable" })
                    ComponentSetValue2(item_cost_component, "stealable", false)
                else
                    EntityAddComponent2(eid, "VariableStorageComponent", { _tags = "ew_try_float" })
                end
                local vel = EntityGetFirstComponentIncludingDisabled(item, "VelocityComponent")
                ComponentSetValue2(vel, "gravity_y", 0)
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

        local immortal = EntityGetFirstComponentIncludingDisabled(item, "LuaComponent", "ew_immortal")
        if immortal ~= 0 then
            EntityRemoveComponent(item, immortal)
        end
        local damage_component = EntityGetFirstComponentIncludingDisabled(item, "DamageModelComponent")
        if damage_component and damage_component ~= 0 then
            ComponentSetValue2(damage_component, "wait_for_kill_flag_on_death", false)
        end

        SetRandomSeed(item + slot_x + item_x, slot_y + item_y)

        local var = EntityGetFirstComponentIncludingDisabled(item, "VariableStorageComponent", "ew_egg")
        if var ~= nil then
            table.insert(wandData, {
                egg = ComponentGetValue2(var, "value_int"),
                slot_x = slot_x,
                slot_y = slot_y,
                active = (mActiveItem == item),
            })
        elseif entity_is_wand(item) then
            table.insert(wandData, {
                data = inventory_helper.serialize_single_item(item),
                slot_x = slot_x,
                slot_y = slot_y,
                active = (mActiveItem == item),
                is_wand = true,
                old_id = item,
            })
        elseif not EntityHasTag(item, "polymorphed_player") then
            table.insert(wandData, {
                data = inventory_helper.serialize_single_item(item),
                slot_x = slot_x,
                slot_y = slot_y,
                active = (mActiveItem == item),
            })
        else
            local data = player_fns.get_player_data_by_local_entity_id(item)
            if data ~= nil then
                local peer_id = data.peer_id
                table.insert(wandData, {
                    peer_id = peer_id,
                    slot_x = slot_x,
                    slot_y = slot_y,
                    active = (mActiveItem == item),
                })
            end
        end
    end

    for _, item in ipairs(inventory_helper.get_inventory_items(player_data, "inventory_full") or {}) do
        local item_comp = EntityGetFirstComponentIncludingDisabled(item, "ItemComponent")
        local slot_x, slot_y = ComponentGetValue2(item_comp, "inventory_slot")
        local item_x, item_y = EntityGetTransform(item)

        local immortal = EntityGetFirstComponentIncludingDisabled(item, "LuaComponent", "ew_immortal")
        if immortal ~= 0 then
            EntityRemoveComponent(item, immortal)
        end
        local damage_component = EntityGetFirstComponentIncludingDisabled(item, "DamageModelComponent")
        if damage_component and damage_component ~= 0 then
            ComponentSetValue2(damage_component, "wait_for_kill_flag_on_death", false)
        end

        SetRandomSeed(item + slot_x + item_x, slot_y + item_y)

        -- local item_id = entity.GetVariable(item, "arena_entity_id")

        -- GlobalsSetValue(tostring(item) .. "_item", tostring(k))
        if entity_is_wand(item) then
            table.insert(spellData, {
                data = inventory_helper.serialize_single_item(item),
                -- id = item_id or (item + Random(1, 10000000)),
                slot_x = slot_x,
                slot_y = slot_y,
                active = (mActiveItem == item),
                is_wand = true,
            })
        else
            table.insert(spellData, {
                data = inventory_helper.serialize_single_item(item),
                -- id = item_id or (item + Random(1, 10000000)),
                slot_x = slot_x,
                slot_y = slot_y,
                active = (mActiveItem == item),
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
    if EntityGetParent(item) ~= 0 then
        EntityRemoveFromParent(item)
    end
    EntityAddChild(entity, item)

    EntitySetComponentsWithTagEnabled(item, "enabled_in_world", false)
    EntitySetComponentsWithTagEnabled(item, "enabled_in_hand", false)
    EntitySetComponentsWithTagEnabled(item, "enabled_in_inventory", true)

    local wand_children = EntityGetAllChildren(item) or {}

    for _, _ in ipairs(wand_children) do
        EntitySetComponentsWithTagEnabled(item, "enabled_in_world", false)
    end
end

local function remove_non_send(item)
    for _, com in ipairs(EntityGetAllComponents(item) or {}) do
        if ComponentHasTag(com, "ew_remove_on_send") then
            EntityRemoveComponent(item, com)
        end
    end
end

local function get_item(itemInfo, inv, player, local_ent)
    --local x, y = EntityGetTransform(player)
    local item_entity
    local item
    if itemInfo.egg ~= nil then
        for _, ent in ipairs(EntityGetWithTag("egg_item")) do
            local var = EntityGetFirstComponentIncludingDisabled(ent, "VariableStorageComponent", "ew_egg")
            if ComponentGetValue2(var, "value_int") == itemInfo.egg then
                item = ent
                break
            end
        end
    elseif itemInfo.is_wand then
        item = inventory_helper.deserialize_single_item(itemInfo.data)
        remove_non_send(item)
        item = EZWand(item)
    elseif itemInfo.peer_id ~= nil then
        item = ctx.players[itemInfo.peer_id].entity
    else
        item = inventory_helper.deserialize_single_item(itemInfo.data)
        remove_non_send(item)
    end

    if item == nil then
        return
    end

    if itemInfo.egg ~= nil then
        pickup_item(inv, item)
        item_entity = item
    elseif itemInfo.is_wand then
        --EntityAddTag(item.entity_id, "ew_client_item")
        item:PickUp(player)
        item_entity = item.entity_id
    elseif itemInfo.peer_id ~= nil then
        pickup_item(inv, item)
        item_entity = item
        np.SetActiveHeldEntity(player, item, false, false)
    else
        --EntityAddTag(item, "ew_client_item")
        pickup_item(inv, item)
        item_entity = item
    end
    ewext.notrack(item_entity)
    local itemComp = EntityGetFirstComponentIncludingDisabled(item_entity, "ItemComponent")
    if itemComp ~= nil then
        ComponentSetValue2(itemComp, "inventory_slot", itemInfo.slot_x, itemInfo.slot_y)
    end
    if not local_ent then
        if itemInfo.egg == nil then
            EntityAddComponent(item_entity, "LuaComponent", {
                script_throw_item = "mods/quant.ew/files/resource/cbs/throw_item.lua",
            })
        end
        local notify = EntityGetFirstComponentIncludingDisabled(item_entity, "LuaComponent", "ew_notify_component")
        if notify ~= nil then
            EntityRemoveComponent(item_entity, notify)
        end
    end
    if itemInfo.active then
        return item_entity
    end
end

function inventory_helper.set_item_data(item_data, player_data, local_ent, has_spells)
    local spells
    if has_spells then
        spells = item_data[2]
        item_data = item_data[1]
    end
    local player = player_data.entity
    if player == nil or not EntityGetIsAlive(player) then
        return
    end

    local inventory2Comp = EntityGetFirstComponentIncludingDisabled(player, "Inventory2Component")
    local inv_quick
    local inv_full
    local children = EntityGetAllChildren(player) or {}
    for _, child in pairs(children) do
        if EntityGetName(child) == "inventory_quick" then
            inv_quick = child
            local inv = EntityGetAllChildren(child)
            if inv ~= nil then
                for _, item in pairs(inv) do
                    if
                        not EntityHasTag(item, "polymorphed_player")
                        and EntityGetFirstComponentIncludingDisabled(item, "VariableStorageComponent", "ew_egg")
                            == nil
                    then
                        EntityKill(item)
                    end
                end
            end
        elseif EntityGetName(child) == "inventory_full" and spells ~= nil then
            inv_full = child
            local inv = EntityGetAllChildren(child)
            if inv ~= nil then
                for _, item in pairs(inv) do
                    EntityKill(item)
                end
            end
        end
    end
    if inv_quick == nil or (inv_full == nil and spells ~= nil) then
        return
    end

    if item_data ~= nil then
        local active_item_entity

        for _, itemInfo in ipairs(item_data) do
            local e = get_item(itemInfo, inv_quick, player, local_ent)
            if e ~= nil then
                active_item_entity = e
            end
        end

        if spells ~= nil then
            for _, itemInfo in ipairs(spells) do
                if itemInfo.slot_x > ComponentGetValue2(inventory2Comp, "full_inventory_slots_x") then
                    ComponentSetValue2(inventory2Comp, "full_inventory_slots_x", itemInfo.slot_x)
                end
                if itemInfo.slot_y > ComponentGetValue2(inventory2Comp, "full_inventory_slots_y") then
                    ComponentSetValue2(inventory2Comp, "full_inventory_slots_y", itemInfo.slot_y)
                end
                get_item(itemInfo, inv_full, player, local_ent)
            end
        end

        if active_item_entity ~= nil then
            np.SetActiveHeldEntity(player, active_item_entity, false, false)
        end
    end
    async(function()
        wait(1)
        inventory2Comp = EntityGetFirstComponentIncludingDisabled(player, "Inventory2Component")
        if inventory2Comp ~= nil then
            ComponentSetValue2(inventory2Comp, "mForceRefresh", true)
        end
    end)
end

local function ensure_notify_component(ent)
    local notify = EntityGetFirstComponentIncludingDisabled(ent, "LuaComponent", "ew_des_lua")
    if notify == nil then
        EntityAddComponent2(ent, "LuaComponent", {
            _tags = "enabled_in_world,enabled_in_hand,enabled_in_inventory,ew_des_lua,ew_remove_on_send",
            script_throw_item = "mods/quant.ew/files/system/entity_sync_helper/item_notify.lua",
        })
    end
end

local last_count = 0

function inventory_helper.has_inventory_changed(player_data)
    local prev_inventory = player_data.prev_inventory_hash
    local inventory_hash = 0
    if
        player_data.entity == nil
        or not EntityGetIsAlive(player_data.entity)
        or GameGetAllInventoryItems(player_data.entity) == nil
    then
        return false
    end

    local items = GameGetAllInventoryItems(player_data.entity) or {}
    for _, item in ipairs(items) do
        if last_count ~= #items then
            ensure_notify_component(item)
            for _, child in ipairs(EntityGetAllChildren(item) or {}) do
                ensure_notify_component(child)
            end
        end
        local item_comp = EntityGetFirstComponentIncludingDisabled(item, "ItemComponent")
        if item_comp ~= nil then
            local slot_x, slot_y = ComponentGetValue2(item_comp, "inventory_slot")
            inventory_hash = (inventory_hash * 19 + (item % 65000 + slot_x + slot_y)) % (math.pow(2, 20) - 1)
        end
    end
    last_count = #items
    player_data.prev_inventory_hash = inventory_hash
    return inventory_hash ~= prev_inventory
end

function inventory_helper.setup_inventory()
    for _, ent in ipairs(inventory_helper.get_all_inventory_items(ctx.my_player) or {}) do
        ensure_notify_component(ent)
    end
end

return inventory_helper
