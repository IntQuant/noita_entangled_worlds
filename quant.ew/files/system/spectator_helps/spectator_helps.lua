local rpc = net.new_rpc_namespace()

local shield_entities = {}

local function delete_shields()
    for _, child in ipairs(EntityGetAllChildren(ctx.my_player.entity) or {}) do
        if EntityGetName(child) == "spectator_shield" then
            EntityKill(child)
        end
    end
end

local shield_angle = rpc:create_var("shield_angle", function(new_value)
    local shield_ent = shield_entities[ctx.rpc_peer_id]
    if shield_ent ~= nil then
        local x, y = EntityGetTransform(shield_ent)
        EntitySetTransform(shield_ent, x, y, new_value)
    end
end)

local function create_shield_for(spectator_peer_id, target_entity)
    if GameGetIsGamepadConnected() then
        return
    end
    local ent = EntityLoad("mods/quant.ew/files/system/spectator_helps/shield_base.xml")
    EntityAddChild(target_entity, ent)
    shield_entities[spectator_peer_id] = ent
    if shield_angle.values[spectator_peer_id] ~= nil then
        local x, y = EntityGetTransform(ent)
        EntitySetTransform(ent, x, y, shield_angle.values[spectator_peer_id])
    end
end

local function maybe_clean_existing_shield(spectator_peer_id)
    if shield_entities[spectator_peer_id] ~= nil then
        EntityKill(shield_entities[spectator_peer_id])
        shield_entities[spectator_peer_id] = nil
    end
end

local spectating_which = rpc:create_var("spectating_which", function(new_value)
    maybe_clean_existing_shield(ctx.rpc_peer_id)
    if ctx.rpc_peer_id ~= new_value and new_value ~= nil then
        create_shield_for(ctx.rpc_peer_id, player_fns.peer_get_player_data(new_value).entity)
    end
end)

local function recreate_shields_of(updated_peer_id)
    for peer_id, target_peer_id in pairs(spectating_which.values) do
        if target_peer_id == updated_peer_id then
            maybe_clean_existing_shield(peer_id)
            create_shield_for(peer_id, player_fns.peer_get_player_data(target_peer_id).entity)
        end
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
    if player_data.status == nil or not player_data.status.is_alive then
        return false
    end
    return true
end

function module.on_local_player_polymorphed(_currently_polymorphed)
    delete_shields()
    recreate_shields_of(ctx.my_id)
end

function module.on_world_update()
    local notplayer_active = GameHasFlagRun("ew_flag_notplayer_active")
    if notplayer_active and ctx.spectating_over_peer_id ~= nil and is_acceptable_help_target(ctx.spectating_over_peer_id) then
        spectating_which.set(ctx.spectating_over_peer_id)
        if GameGetFrameNum() % 6 == 3 then
            local x,y = DEBUG_GetMouseWorld()
            local cx, cy = GameGetCameraPos()
            local dx, dy = x - cx, y - cy
            local angle = math.atan2(dy, dx)
            shield_angle.set(angle)
        end
    else
        spectating_which.set(nil)
    end
end

function module.on_local_player_spawn()
    -- Cleanup after restarts
    delete_shields()
end

function module.on_client_polymorphed(peer_id, _player_data)
    recreate_shields_of(peer_id)
end

function module.on_client_spawned(peer_id, _new_playerdata)
    recreate_shields_of(peer_id)
end

return module