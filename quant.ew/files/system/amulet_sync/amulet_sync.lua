local rpc = net.new_rpc_namespace()

local cosmetics_states = {}
local module = {}

local function enable_cosmetics(player_data)
    local entity_id = player_data.entity
    local amulet, amulet_gem, hat = unpack(cosmetics_states[player_data.peer_id] or {false, false, false})
    if amulet then
        EntitySetComponentsWithTagEnabled(entity_id, "player_amulet", true)
    end
    if amulet_gem then
        EntitySetComponentsWithTagEnabled( entity_id, "player_amulet_gem", true )
    end
    if hat then
        EntitySetComponentsWithTagEnabled( entity_id, "player_hat2", true )
    end
end

function module.on_should_send_updates()
    rpc.send_cosmetics_state(HasFlagPersistent( "secret_amulet" ), HasFlagPersistent( "secret_amulet_gem" ), HasFlagPersistent( "secret_hat" ))
end

function module.on_client_spawned(peer_id, player_data)
    enable_cosmetics(player_data)
end

function rpc.send_cosmetics_state(amulet, amulet_gem, hat)
    cosmetics_states[ctx.rpc_peer_id] = {amulet, amulet_gem, hat}
    enable_cosmetics(ctx.rpc_player_data)
end

return module