ModLuaFileAppend("data/scripts/biomes/temple_shared.lua", "mods/quant.ew/files/system/stevari/append.lua")

local function request_flag_slow(x, y)
    local flag = "ew_spawn_stevari" .. ":" .. math.floor(x / 1024 + 0.5) .. ":" .. math.floor(y / 1024 + 0.5)
    net.send_flags("3" .. math.floor(x) .. " " .. math.floor(y) .. " " .. flag)
end

util.add_cross_call("ew_spawn_stevari", function(x, y)
    request_flag_slow(x, y)
end)

return {}
