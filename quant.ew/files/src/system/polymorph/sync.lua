local util = dofile_once("mods/quant.ew/files/src/util.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local np = require("noitapatcher")

local rpc = net.new_rpc_namespace()

np.CrossCallAdd("ew_player_polymorphing_to", function(ent)
    player_fns.replace_player_entity(ent, ctx.my_player)
end)

local module = {}

function module.on_local_player_spawn(my_player)
    EntityAddComponent2(my_player.entity, "LuaComponent", {script_polymorphing_to = "mods/quant.ew/files/src/system/polymorph/cbs/polymorphing_to.lua"})
end

return module