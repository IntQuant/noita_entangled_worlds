local util = dofile_once("mods/quant.ew/files/src/util.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local np = require("noitapatcher")

local rpc = net.new_rpc_namespace()

local module = {}

function module.on_world_update_post()
    local ent = np.GetPlayerEntity()
    if ent ~= ctx.my_player.entity then
        player_fns.replace_player_entity(ent, ctx.my_player)
    end
end

return module
