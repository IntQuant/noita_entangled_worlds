local rpc = net.new_rpc_namespace()

local shield_entities = {}

local function create_shield_for(spectator_peer_id, target_entity)
    local ent = EntityLoad("mods/quant.ew/files/system/spectator_helps/shield_base.xml")
    EntityAddChild(target_entity, ent)
    shield_entities[spectator_peer_id] = ent
    
    -- EntityAddComponent2(ent, "EnergyShieldComponent", {})
end

local spectating_which = rpc:create_var("spectating_which", function(new_value)
    if shield_entities[ctx.rpc_peer_id] ~= nil then
        EntityKill(shield_entities[ctx.rpc_peer_id])
        shield_entities[ctx.rpc_peer_id] = nil
    end
    if ctx.rpc_peer_id ~= new_value and new_value ~= nil then
        create_shield_for(ctx.rpc_peer_id, player_fns.peer_get_player_data(new_value).entity)
    end
end)

local shield_angle = rpc:create_var("shield_angle", function(new_value)
    local shield_ent = shield_entities[ctx.rpc_peer_id]
    if shield_ent ~= nil then
        local x, y = EntityGetTransform(shield_ent)
        EntitySetTransform(shield_ent, x, y, new_value)
    end
end)

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

function module.on_world_update()
    local notplayer_active = GameHasFlagRun("ew_flag_notplayer_active")
    if notplayer_active and ctx.spectating_over_peer_id ~= nil and is_acceptable_help_target(ctx.spectating_over_peer_id) then
        spectating_which.set(ctx.spectating_over_peer_id)
        if GameGetFrameNum() % 6 == 3 then
            local mx, my = DEBUG_GetMouseWorld()
            local cx, cy = GameGetCameraPos()
            local dx, dy = mx - cx, my - cy
            local angle = math.atan2(dy, dx)
            shield_angle.set(angle)
        end
    else
        spectating_which.set(nil)
    end
end

function module.on_local_player_spawn()
    -- Cleanup after restarts
    for _, child in ipairs(EntityGetAllChildren(ctx.my_player.entity) or {}) do
        if EntityGetName(child) == "spectator_shield" then
            EntityKill(child)
        end
    end
end

return module