local rpc = net.new_rpc_namespace()

local shield_entities = {}

rpc.opts_everywhere()
function rpc.add_shield(target)
    if GameHasFlagRun("ending_game_completed") then
        return
    end
    local entity = ctx.players[target].entity
    if not EntityGetIsAlive(entity) or EntityHasTag(entity, "polymorphed") then
        return
    end
    local ent = EntityLoad("mods/quant.ew/files/system/spectator_helps/shield_base.xml")
    EntityAddChild(entity, ent)
    shield_entities[ctx.rpc_peer_id] = {target, ent}
end

rpc.opts_everywhere()
function rpc.del_shield(id)
    if shield_entities[id] ~= nil then
        EntityKill(shield_entities[id][2])
        shield_entities[id] = nil
    end
end

local module = {}

local function is_acceptable_help_target(spectating_over)
    -- No helping myself
    if spectating_over == ctx.my_id then
        return false
    end
    -- No helping notplayers
    local player_data = ctx.players[spectating_over]
    if player_data == nil or player_data.status == nil or not player_data.status.is_alive then
        return false
    end
    -- No helping polied players
    if EntityHasTag(player_data.entity, "polymorphed") then
        return false
    end
    if shield_entities[ctx.my_id] ~= nil then
        return false
    end
    return true
end

local function kill_all_shields()
    for _, child in ipairs(EntityGetAllChildren(ctx.my_player.entity) or {}) do
        if EntityGetName(child) == "spectator_shield" then
            EntityKill(child)
        end
    end
end

function module.on_local_player_spawn()
    -- Cleanup after restarts
    kill_all_shields()
end

function module.on_local_player_polymorphed(_currently_polymorphed)
    kill_all_shields()
end

function rpc.send_money(money)
    local entity = ctx.rpc_player_data.entity
    local wallet = EntityGetFirstComponentIncludingDisabled(entity, "WalletComponent")
    ComponentSetValue2(wallet, "money", money)
end

local last_spectate

function module.on_world_update()
    if GameHasFlagRun("ending_game_completed") then
        return
    end
    if GameGetFrameNum() % 10 == 8 then
        local notplayer_active = GameHasFlagRun("ew_flag_notplayer_active")
        if notplayer_active and ctx.spectating_over_peer_id ~= nil and is_acceptable_help_target(ctx.spectating_over_peer_id) then
            rpc.add_shield(ctx.spectating_over_peer_id)
        elseif last_spectate ~= nil and last_spectate ~= ctx.spectating_over_peer_id then
            rpc.del_shield(ctx.my_id)
        end
        last_spectate = ctx.spectating_over_peer_id
    end
    for peer_id, _ in pairs(ctx.players) do
        for shield_id, shield in pairs(shield_entities) do
            if peer_id == shield[1] then
                local shield_ent = shield[2]
                local my_x, my_y = EntityGetTransform(ctx.players[peer_id].entity)
                local his_x, his_y
                if shield_id == ctx.my_id then
                    if GameGetIsGamepadConnected() then
                        his_x, his_y = InputGetJoystickAnalogStick(0, 1)
                        his_x, his_y = his_x * 60 + my_x, his_y * 60 + my_y
                    else
                        his_x, his_y = DEBUG_GetMouseWorld()
                    end
                else
                    for _, child in ipairs(EntityGetAllChildren(ctx.players[shield_id].entity) or {}) do
                        if (EntityGetName(child) == "cursor") then
                            his_x, his_y = EntityGetTransform(child)
                            break
                        end
                    end
                end
                if his_x == nil then
                    goto continue
                end
                local dx, dy = his_x - my_x, his_y - my_y
                if dx * dx + dy * dy > 350 * 350 then
                    goto continue
                end
                local angle = math.atan2(dy, dx)
                local x, y = EntityGetTransform(shield_ent)
                EntitySetTransform(shield_ent, x, y, angle)
            end
        end
        ::continue::
    end
    if GameGetFrameNum() % 60 == 47 then
        local wallet = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "WalletComponent")
        rpc.send_money(ComponentGetValue2(wallet, "money"))
    end
end

return module