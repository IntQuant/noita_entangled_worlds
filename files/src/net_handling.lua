local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local enemy_sync = dofile_once("mods/quant.ew/files/src/enemy_sync.lua")
local world_sync = dofile_once("mods/quant.ew/files/src/world_sync.lua")
local perk_fns = dofile_once("mods/quant.ew/files/src/perk_fns.lua")

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
    local input_data = value.i
    local pos_data = value.p
    local slot_data = value.s
    -- GamePrint("Player update for "..peer_id.." "..pos_data.x.." "..pos_data.y)
    if not player_fns.peer_has_player(peer_id) then
        player_fns.spawn_player_for(peer_id, pos_data.x, pos_data.y)
    end
    local player_data = player_fns.peer_get_player_data(peer_id)
    player_fns.deserialize_inputs(input_data, player_data)
    player_fns.deserialize_position(pos_data, player_data)
    if slot_data ~= nil then
        player_fns.set_current_slot(slot_data, player_data)
    end
end

function net_handling.mod.inventory(peer_id, inventory_state)
    if not player_fns.peer_has_player(peer_id) then
        return
    end
    local player_data = player_fns.peer_get_player_data(peer_id)
    player_fns.deserialize_items(inventory_state, player_data)
    GamePrint("synced inventory")
end

function net_handling.mod.perks(peer_id, perk_data)
    if not player_fns.peer_has_player(peer_id) then
        return
    end
    local player_data = player_fns.peer_get_player_data(peer_id)
    perk_fns.update_perks(perk_data, player_data)
end

function net_handling.mod.enemy(peer_id, enemy_data)
    if peer_id == ctx.host_id then
        enemy_sync.handle_enemy_data(enemy_data)
    end
end

function net_handling.mod.world(peer_id, world_data)
    if peer_id == ctx.host_id then
        world_sync.handle_world_data(world_data)
    end
end

return net_handling