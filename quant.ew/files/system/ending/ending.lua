local util = dofile_once("mods/quant.ew/files/core/util.lua")
local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")
local np = require("noitapatcher")

local rpc = net.new_rpc_namespace()

local module = {}

util.replace_text_in("data/entities/animals/boss_centipede/sampo.xml", "data/entities/animals/boss_centipede/ending/sampo_start_ending_sequence.lua", "mods/quant.ew/files/system/ending/ending_sequence.lua")

rpc.opts_reliable()
rpc.opts_everywhere()
function rpc.gather_and_do_ending(x, y, sx, sy)
    EntitySetTransform(ctx.my_player.entity, x, y)

    local entity = EntityCreateNew("totally_sampo")
    EntitySetTransform(entity, sx, sy)

    local old_updated = GetUpdatedEntityID
    function GetUpdatedEntityID()
        return entity
    end

    dofile("data/entities/animals/boss_centipede/ending/sampo_start_ending_sequence.lua")

    GetUpdatedEntityID = old_updated
end

np.CrossCallAdd("ew_ending_sequence", function(sx, sy)
    local x, y = EntityGetTransform(ctx.my_player.entity)
    rpc.gather_and_do_ending(x, y, sx, sy)
end)

return module