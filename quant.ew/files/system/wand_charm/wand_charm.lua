local rpc = net.new_rpc_namespace()
ModLuaFileAppend("data/scripts/animals/wand_charm.lua", "mods/quant.ew/files/system/wand_charm/append.lua")
rpc.opts_reliable()
function rpc.charm(gid)
    local item = ewext.find_by_gid(gid)
    if item ~= nil then
        EntityAddComponent2(
            item,
            "LuaComponent",
            { script_source_file = "mods/quant.ew/files/system/wand_charm/charm.lua", remove_after_executed = true }
        )
    end
end
util.add_cross_call("ew_charm_sync", function(id)
    local gid
    for _, v in ipairs(EntityGetComponent(id, "VariableStorageComponent") or {}) do
        if ComponentGetValue2(v, "name") == "ew_gid_lid" then
            gid = v
            break
        end
    end
    if gid ~= nil then
        rpc.charm(ComponentGetValue2(gid, "value_string"))
    end
end)
return {}
