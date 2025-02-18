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
    local dead = GameHasFlagRun("ew_flag_notplayer_active")
    local polied = EntityHasTag(ctx.my_player.entity, "polymorphed_player")
    local cess = EntityHasTag(ctx.my_player.entity, "polymorphed_cessation")
        and (not ctx.proxy_opt.perma_death or not dead)
    local a = 0
    if dead then
        a = 1
    end
    local b = 0
    if polied then
        b = 1
    end
    local c = 0
    if cess then
        c = 1
    end
    net.proxy_send(
        "cam_pos",
        math.floor(x)
            .. " "
            .. math.floor(y)
            .. " "
            .. math.floor(mx)
            .. " "
            .. math.floor(my)
            .. " "
            .. ptt
            .. " "
            .. a
            .. " "
            .. b
            .. " "
            .. c
    )
end

return module
