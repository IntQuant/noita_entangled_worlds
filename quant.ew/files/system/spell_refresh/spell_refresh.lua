util.add_cross_call("ew_refresh_inventory", function()
    local inventory_state = player_fns.serialize_items(ctx.my_player)
    if inventory_state ~= nil then
        net.send_player_inventory(inventory_state)
    end
end)

ModLuaFileAppend("data/scripts/items/spell_refresh.lua", "mods/quant.ew/files/system/spell_refresh/append.lua")

return {}