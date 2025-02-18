local module = {}

function module.on_world_update()
    if GameGetFrameNum() % 4 ~= 2 then
        return
    end
    local rebind = tonumber(ModSettingGet("quant.ew.rebind_ptt") or 42)
    local ptt = 0
    if (rebind == 42 and InputIsMouseButtonDown(3)) or (rebind ~= 42 and InputIsKeyDown(rebind)) then
        ptt = 1
    end
    local x, y = GameGetCameraPos()
    local mx, my = EntityGetTransform(ctx.my_player.entity)
    net.proxy_send(
        "cam_pos",
        math.floor(x) .. " " .. math.floor(y) .. " " .. math.floor(mx) .. " " .. math.floor(my) .. " " .. ptt
    )
end

return module
