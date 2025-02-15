local module = {}

function module.on_world_update()
    if GameGetFrameNum() % 4 ~= 2 then
        return
    end
    for peer_id, player_data in pairs(ctx.players) do
        local x, y = EntityGetTransform(player_data.entity)
        if x ~= nil and y ~= nil then
            net.proxy_send("peer_pos", peer_id .. " " .. math.floor(x) .. " " .. math.floor(y))
        end
    end
    local rebind = tonumber(ModSettingGet("quant.ew.rebind_ptt") or 42)
    local ptt = 0
    if (rebind == 42 and InputIsMouseButtonDown(3)) or (rebind ~= 42 and InputIsKeyDown(rebind)) then
        ptt = 1
    end
    GamePrint(ptt)
    net.proxy_send("ptt", ptt)
end

return module
