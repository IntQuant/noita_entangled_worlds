local module = {}

local ptt = 0

function module.on_world_update()
    if GameGetFrameNum() % 4 ~= 2 then
        return
    end
    local rebind = tonumber(ModSettingGet("quant.ew.rebind_ptt") or 42)
    if ModSettingGet("quant.ew.ptt_toggle") then
        if InputIsKeyJustDown(rebind) then
            if ptt == 1 then
                ptt = 0
            else
                ptt = 1
            end
        end
    else
        ptt = 0
        if InputIsKeyDown(rebind) then
            ptt = 1
        end
    end
    local x, y = GameGetCameraPos()
    local mx, my = EntityGetTransform(ctx.my_player.entity)
    if mx == nil then
        return
    end
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
    local is_dead = 0
    if GameHasFlagRun("ew_flag_notplayer_active") then
        is_dead = 1
    end
    local d = 0
    if ctx.proxy_opt.no_notplayer or ctx.proxy_opt.perma_death then
        d = 1
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
            .. is_dead
            .. " "
            .. d
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
