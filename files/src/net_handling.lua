local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")

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
    ctx.my_id = tonumber(value)
end

function net_handling.mod.player(peer_id, value)
    local input_data = value.inp
    local pos_data = value.pos
    -- GamePrint("Player update for "..peer_id.." "..pos_data.x.." "..pos_data.y)
    if not player_fns.peer_has_player(peer_id) then
        player_fns.spawn_player_for(peer_id, pos_data.x, pos_data.y)
    end
    local player_data = player_fns.peer_get_player_data(peer_id)
    player_fns.deserialize_inputs(input_data, player_data)
    player_fns.deserialize_position(pos_data, player_data)
end

function net_handling.mod.inventory(peer_id, inventory_state)
    if not player_fns.peer_has_player(peer_id) then
        return
    end
    local player_data = player_fns.peer_get_player_data(peer_id)
    player_fns.deserialize_items(inventory_state, player_data)
    GamePrint("synced inventory")
end

return net_handling