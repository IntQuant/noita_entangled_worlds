local perk_fns = dofile_once("mods/quant.ew/files/core/perk_fns.lua")

local net_handling = {
    proxy = {},
    mod = {},
}

function net_handling.proxy.seed(_, value)
    local seed = tonumber(value)
    if seed == 0 then
        seed = 1
    end
    if seed ~= nil then
        SetWorldSeed(seed)
        SetRandomSeed(seed, 141)
    end
end

function net_handling.proxy.peer_id(_, value)
    print("My peer_id: " .. value)
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

function net_handling.proxy.proxy_opt(_, key, value)
    print("Proxy opt [str]: " .. key .. " = " .. tostring(value))
    ctx.proxy_opt[key] = value
end

function net_handling.proxy.proxy_opt_num(_, key, value)
    ctx.proxy_opt[key] = tonumber(value)
end

function net_handling.proxy.proxy_opt_bool(_, key, value)
    print("Proxy opt [bool]: " .. key .. " = " .. value)
    ctx.proxy_opt[key] = value == "true"
end

function net_handling.proxy.dc(_, peer_id)
    local player = ctx.players[peer_id]
    if
        player == nil
        or player.entity == nil
        or (EntityHasTag(player.entity, "ew_notplayer") and (ctx.proxy_opt.no_notplayer or ctx.proxy_opt.perma_death))
    then
        return
    end
    local sprite = EntityGetFirstComponentIncludingDisabled(player.entity, "SpriteComponent")
    local name = ComponentGetValue2(sprite, "image_file")
    local new = string.sub(name, 0, -5) .. "_dc.xml"
    ComponentSetValue2(sprite, "image_file", new)
    for _, child in ipairs(EntityGetAllChildren(player.entity) or {}) do
        if EntityGetName(child) == "notcursor" or EntityGetName(child) == "cursor" then
            sprite = EntityGetFirstComponentIncludingDisabled(child, "SpriteComponent")
            EntitySetComponentIsEnabled(child, sprite, false)
        end
    end
    ctx.players[peer_id].dc = true
    EntityRemoveTag(player.entity, "ew_client")
    EntityRemoveTag(player.entity, "ew_peer")
end

function net_handling.proxy.leave(_, peer_id)
    local player = ctx.players[peer_id]
    if player ~= nil then
        GamePrint("Player " .. player.name .. " left")
        EntityKill(player.entity)
        ctx.players[peer_id] = nil
    else
        GamePrint("Player " .. peer_id .. " left")
    end
end

function net_handling.mod.inventory(peer_id, inventory_state)
    if not player_fns.peer_has_player(peer_id) then
        return
    end
    local player_data = player_fns.peer_get_player_data(peer_id)
    player_data.latest_inventory = inventory_state[1]
    player_fns.deserialize_items(inventory_state, player_data, true)
end

function net_handling.mod.perks(peer_id, perk_data)
    if not player_fns.peer_has_player(peer_id) then
        return
    end
    local player_data = player_fns.peer_get_player_data(peer_id)
    local comp = EntityGetFirstComponent(player_data.entity, "VariableStorageComponent", "ew_current_perks")
    if comp ~= nil and comp ~= 0 then
        perk_fns.update_perks(perk_data, player_data)
    end
end

local function reset_cast_state_if_has_any_other_item(player_data)
    local inventory2Comp = EntityGetFirstComponentIncludingDisabled(player_data.entity, "Inventory2Component")
    if inventory2Comp == nil then
        return
    end
    local mActiveItem = ComponentGetValue2(inventory2Comp, "mActiveItem")

    for k, item in ipairs(inventory_helper.get_inventory_items(player_data, "inventory_quick") or {}) do
        if item ~= mActiveItem then
            np.SetActiveHeldEntity(player_data.entity, item)
            np.SetActiveHeldEntity(player_data.entity, mActiveItem)
            break
        end
    end
end

function net_handling.mod.fire(peer_id, fire_data)
    local player_data = player_fns.peer_get_player_data(peer_id)
    local entity = player_data.entity
    if not EntityGetIsAlive(entity) then
        return
    end
    local rng = fire_data[1]
    local message = fire_data[2]

    local switched_now = fire_data.switched_now == true
    if switched_now then
        reset_cast_state_if_has_any_other_item(player_data)
    end

    GlobalsSetValue("ew_shooter_rng_" .. tostring(peer_id), tostring(message.special_seed))

    GlobalsSetValue("ew_action_rng_" .. tostring(peer_id), tostring(message.player_action_rng))

    player_data.projectile_rng_init = rng

    local controlsComp = EntityGetFirstComponentIncludingDisabled(entity, "ControlsComponent")

    if controlsComp ~= nil then
        local inventory2Comp = EntityGetFirstComponentIncludingDisabled(entity, "Inventory2Component")

        if inventory2Comp == nil then
            return
        end

        local mActiveItem = ComponentGetValue2(inventory2Comp, "mActiveItem")

        if mActiveItem ~= nil then
            local ability = EntityGetFirstComponentIncludingDisabled(mActiveItem, "AbilityComponent")
            if
                EntityHasTag(mActiveItem, "card_action")
                or (ability ~= nil and ComponentGetValue2(ability, "use_gun_script"))
            then
                local aimNormal_x, aimNormal_y = ComponentGetValue2(controlsComp, "mAimingVectorNormalized")
                local aim_x, aim_y = ComponentGetValue2(controlsComp, "mAimingVector")
                local firing = ComponentGetValue2(controlsComp, "mButtonDownFire")
                if message.mana ~= -1 then
                    ComponentSetValue2(ability, "mana", message.mana)
                end

                ComponentSetValue2(controlsComp, "mButtonDownFire", false)

                local wand_x, wand_y, wand_r = message.x, message.y, message.r

                local x = wand_x + (aimNormal_x * 2)
                local y = wand_y + (aimNormal_y * 2)
                y = y - 1

                local target_x = x + aim_x
                local target_y = y + aim_y

                util.set_ent_firing_blocked(entity, false)

                EntityAddTag(entity, "player_unit")
                np.UseItem(entity, mActiveItem, true, true, true, x, y, target_x, target_y)
                EntityRemoveTag(entity, "player_unit")

                util.set_ent_firing_blocked(entity, true)

                ComponentSetValue2(controlsComp, "mButtonDownFire", firing)
            end
        end
    end
    if #player_data.projectile_rng_init > 0 then
        -- Shouldn't happen
    end
end

function net_handling.mod.welcome(peer_id, _)
    ctx.events.new_player_just_connected = true
    if not ctx.run_ended then
        ctx.hook.on_should_send_updates()
    end
end

function net_handling.mod.join_notify(_, name)
    if ctx.run_ended then
        if name[2] == "" then
            name[2] = "new run"
        end
        GamePrint(name[1] .. " started gamemode " .. name[2])
    end
end

function net_handling.proxy.end_run(_, _)
    local entity = ctx.my_player.entity
    local damage = ComponentGetValue2(entity, "DamageModelComponent")
    if damage ~= nil then
        ComponentSetValue2(damage, "ui_report_damage", false)
        ComponentSetValue2(damage, "hp", 2 ^ -38)
    end
    if entity ~= nil and EntityGetIsAlive(entity) and not EntityHasTag(entity, "ew_notplayer") then
        EntityInflictDamage(entity, 1000000, "DAMAGE_CURSE", "Run Ended", "NONE", 0, 0, GameGetWorldStateEntity())
    end
    ctx.run_ended = true
    EntityKill(entity)
    GameTriggerGameOver()
end

return net_handling
