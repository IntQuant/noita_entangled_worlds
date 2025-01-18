local rpc = net.new_rpc_namespace()

ModLuaFileAppend("data/scripts/items/gold_orb.lua", "mods/quant.ew/files/system/shiny_orb/append.lua")

ModLuaFileAppend("data/scripts/items/gold_orb_greed.lua", "mods/quant.ew/files/system/shiny_orb/append_greed.lua")

rpc.opts_everywhere()
function rpc.kicked_orb(gid, rx, ry, greed)
    local entity_id = ewext.find_by_gid(gid)
    if entity_id == nil then
        return
    end
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
    local x, y = EntityGetTransform(entity)
    local gid = EntityGetFirstComponentIncludingDisabled(id, "VariableStorageComponent", "ew_gid_lid")
    if gid ~= nil then
        rpc.kicked_orb(ComponentGetValue2(gid, "value_string"), x + entity, y - GameGetFrameNum(), greed)
    end
end)

return {}
