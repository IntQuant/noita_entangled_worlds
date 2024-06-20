local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local util = dofile_once("mods/quant.ew/files/src/util.lua")
local perk_fns = dofile_once("mods/quant.ew/files/src/perk_fns.lua")
local inventory_helper = dofile_once("mods/quant.ew/files/src/inventory_helper.lua")
local item_sync = dofile_once("mods/quant.ew/files/src/item_sync.lua")

local np = require("noitapatcher")

local net_handling = {
    proxy = {},
    mod = {},
}

function net_handling.proxy.seed(_, value)
    local seed = tonumber(value)
        if seed ~= nil then
        SetWorldSeed(seed)
        SetRandomSeed(seed, 141)
    end
end

function net_handling.proxy.peer_id(_, value)
    print("My peer_id: "..value)
    ctx.my_id = value
    ctx.is_host = ctx.my_id == ctx.host_id
end

function net_handling.proxy.debug(_, value)
    ctx.debug = value == "true"
end

function net_handling.proxy.host_id(_, value)
    ctx.host_id = value
    ctx.is_host = ctx.my_id == ctx.host_id
end

function net_handling.proxy.name(_, value)
    print("Got name from proxy: "..value)
    ctx.my_name = value
end

function net_handling.proxy.proxy_opt(_, key, value)
    print("Proxy opt: "..key.." = "..value)
    ctx.proxy_opt[key] = value
end

function net_handling.mod.player(peer_id, value)
    local input_data = value.i
    local pos_data = value.p
    local slot_data = value.s
    -- GamePrint("Player update for "..peer_id.." "..pos_data.x.." "..pos_data.y)
    if not player_fns.peer_has_player(peer_id) then
        player_fns.spawn_player_for(peer_id, pos_data.x, pos_data.y)
    end
    local player_data = player_fns.peer_get_player_data(peer_id)
    if input_data ~= nil then
        player_fns.deserialize_inputs(input_data, player_data)
    end
    if pos_data ~= nil then
        player_fns.deserialize_position(pos_data, player_data)
    end
    if slot_data ~= nil then
        player_fns.set_current_slot(slot_data, player_data)
    end
end

function net_handling.mod.inventory(peer_id, inventory_state)
    if not player_fns.peer_has_player(peer_id) then
        return
    end
    local player_data = player_fns.peer_get_player_data(peer_id)
    player_data.latest_inventory = inventory_state
    player_fns.deserialize_items(inventory_state, player_data)
    -- GamePrint("synced inventory")
end

function net_handling.mod.perks(peer_id, perk_data)
    if not player_fns.peer_has_player(peer_id) then
        return
    end
    local player_data = player_fns.peer_get_player_data(peer_id)
    perk_fns.update_perks(perk_data, player_data)
end

function net_handling.mod.fire(peer_id, fire_data)
    local rng = fire_data[1]
    local message = fire_data[2]
    local player_data = player_fns.peer_get_player_data(peer_id)
    local entity = player_data.entity
    
    GlobalsSetValue("ew_shooter_rng_" .. tostring(peer_id), tostring(message.special_seed))
                
    GlobalsSetValue("ew_action_rng_"..tostring(peer_id), tostring(message.player_action_rng))

    player_data.projectile_rng_init = rng

    local controlsComp = EntityGetFirstComponentIncludingDisabled(entity, "ControlsComponent")

    if controlsComp ~= nil then
        local inventory2Comp = EntityGetFirstComponentIncludingDisabled(entity, "Inventory2Component")

        if (inventory2Comp == nil) then
            return
        end

        local mActiveItem = ComponentGetValue2(inventory2Comp, "mActiveItem")

        if mActiveItem ~= nil then
            local aimNormal_x, aimNormal_y = ComponentGetValue2(controlsComp, "mAimingVectorNormalized")
            local aim_x, aim_y = ComponentGetValue2(controlsComp, "mAimingVector")
            local firing = ComponentGetValue2(controlsComp, "mButtonDownFire")

            ComponentSetValue2(controlsComp, "mButtonDownFire", false)

            local wand_x, wand_y, wand_r = message.x, message.y, message.r

            local x = wand_x + (aimNormal_x * 2)
            local y = wand_y + (aimNormal_y * 2)
            y = y - 1

            local target_x = x + aim_x
            local target_y = y + aim_y

            util.set_ent_firing_blocked(entity, false)

            np.UseItem(entity, mActiveItem, true, true, true, x, y, target_x, target_y)

            util.set_ent_firing_blocked(entity, true)

            ComponentSetValue2(controlsComp, "mButtonDownFire", firing)
        end
    end
    if #player_data.projectile_rng_init > 0 then
        -- GamePrint("unused projectile_rng_init values left "..#player_data.projectile_rng_init)
    end
end

function net_handling.mod.item_global(peer_id, item_data)
    if peer_id ~= ctx.host_id then
        return
    end
    local item = inventory_helper.deserialize_single_item(item_data)
    EntityAddTag(item, "ew_global_item")
    item_sync.ensure_notify_component(item)
    -- GamePrint("Got global item: "..item)
    local g_id = EntityGetFirstComponentIncludingDisabled(item, "VariableStorageComponent", "ew_global_item_id")
    if g_id == nil then
        EntityAddComponent2(item, "VariableStorageComponent", {
            _tags = "ew_global_item_id",
            value_int = item_data.g_id
        })
    else
        ComponentSetValue2(g_id, "value_int", item_data.g_id)
    end
end

function net_handling.mod.item_localize(peer_id, localize_data)
    local l_peer_id = localize_data[1]
    local item_id = localize_data[2]
    -- GamePrint("Localize "..item_id.." to "..l_peer_id)
    if l_peer_id ~= ctx.my_id then
        item_sync.remove_item_with_id(item_id)
    end
end

function net_handling.mod.item_localize_req(peer_id, gid)
    if not ctx.is_host then
        return
    end
    item_sync.host_localize_item(gid, peer_id)
end

function net_handling.mod.item_upload(peer_id, item_data)
    if not ctx.is_host then
        return
    end
    item_sync.upload(item_data)
end

function net_handling.mod.welcome(peer_id, _)
    ctx.events.new_player_just_connected = true
    ctx.hook.on_should_send_updates()
end

return net_handling