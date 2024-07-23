local util = dofile_once("mods/quant.ew/files/src/util.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local np = require("noitapatcher")

local rpc = net.new_rpc_namespace()

local module = {}

util.replace_text_in("data/entities/animals/boss_centipede/sampo.xml", "data/entities/animals/boss_centipede/ending/sampo_start_ending_sequence.lua", "mods/quant.ew/files/src/system/ending/ending_sequence.lua")

rpc.opts_reliable()
rpc.opts_everywhere()
function rpc.gather_and_do_ending(x, y)
    EntitySetTransform(ctx.my_player.entity, x, y)
    dofile("data/entities/animals/boss_centipede/ending/sampo_start_ending_sequence.lua")
end

np.CrossCallAdd("ew_ending_sequence", function()
    local x, y = EntityGetTransform(ctx.my_player.entity)
    rpc.gather_and_do_ending(x, y)
end)

return module
