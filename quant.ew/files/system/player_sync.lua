local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/core/player_fns.lua")

local rpc = net.new_rpc_namespace()

local module = {}

function rpc.player_update(input_data, pos_data, current_slot, team)
    local peer_id = ctx.rpc_peer_id

    if not player_fns.peer_has_player(peer_id) then
        player_fns.spawn_player_for(peer_id, pos_data.x, pos_data.y, team)
    end
    local player_data = player_fns.peer_get_player_data(peer_id)

    if team ~= nil and not GameHasFlagRun("ending_game_completed") then
        local my_team = ctx.proxy_opt.friendly_fire_team - 1
        if my_team ~= -1 and team ~= -1 and (team == 0 or my_team == 0 or team ~= my_team) then
            GenomeSetHerdId(player_data.entity, "player_pvp")
        else
            GenomeSetHerdId(player_data.entity, "player")
        end
    end


    if input_data ~= nil then
        player_fns.deserialize_inputs(input_data, player_data)
    end
    if pos_data ~= nil then
        player_fns.deserialize_position(pos_data, player_data)
    end
    if current_slot ~= nil then
        player_fns.set_current_slot(current_slot, player_data)
    end
end

function rpc.check_gamemode(gamemode)
    local mn = np.GetGameModeNr()
    local gm = np.GetGameModeName(mn)
    if gamemode ~= gm then
        GamePrint("Player: " .. ctx.rpc_player_data.name .. ", is on a different gamemode number then you")
        GamePrint("his game mode: ".. gamemode)
        GamePrint("your game mode: ".. gm)
    end
end

function module.on_world_update()
    local input_data = player_fns.serialize_inputs(ctx.my_player)
    local pos_data =  player_fns.serialize_position(ctx.my_player)
    local current_slot = player_fns.get_current_slot(ctx.my_player)
    if input_data ~= nil and pos_data ~= nil then
        local my_team
        if ctx.proxy_opt.friendly_fire and GameGetFrameNum() % 60 == 43 then
            my_team = ctx.proxy_opt.friendly_fire_team - 1
        end

        rpc.player_update(input_data, pos_data, current_slot, my_team)
        if GameGetFrameNum() % 120 == 0 then
            local n = np.GetGameModeNr()
            rpc.check_gamemode(np.GetGameModeName(n))
        end
    end
end

return module