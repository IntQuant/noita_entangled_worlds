local item_sync = dofile_once("mods/quant.ew/files/system/item_sync.lua")

local rpc = net.new_rpc_namespace()

ModLuaFileAppend("data/scripts/items/gold_orb.lua", "mods/quant.ew/files/system/shiny_orb/append.lua")

ModLuaFileAppend("data/scripts/items/gold_orb_greed.lua", "mods/quant.ew/files/system/shiny_orb/append_greed.lua")

rpc.opts_everywhere()
function rpc.kicked_orb(gid, rx, ry, greed)
    if gid == nil or item_sync.find_by_gid(gid) == nil then
        return
    end
    local entity_id = item_sync.find_by_gid(gid)
    local old = SetRandomSeed
    function SetRandomSeed()
        old(rx, ry)
    end
    function GetUpdatedEntityID()
        return entity_id
    end
    if greed then
        dofile("data/scripts/items/gold_orb_greed.lua")
    else
        dofile("data/scripts/items/gold_orb.lua")
    end
    drop()
end

util.add_cross_call("ew_kicked_orb", function(entity, entity_who_kicked, greed)
    if entity_who_kicked ~= ctx.my_player.entity then
        return
    end
    local x, y = EntityGetTransform( entity )
    rpc.kicked_orb(item_sync.get_global_item_id(entity), x + entity, y - GameGetFrameNum(), greed)
end)

return {}