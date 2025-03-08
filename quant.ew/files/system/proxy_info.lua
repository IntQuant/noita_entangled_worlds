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

function module.on_world_update_host()
    if GameGetFrameNum() % 8 ~= 2 then
        return
    end
    local s = ""
    for peer, data in pairs(ctx.players) do
        local x, y = EntityGetTransform(data.entity)
        if x == nil then
            return
        end
        s = s .. " " .. tostring(peer) .. " " .. math.floor(x) .. " " .. math.floor(y)
    end
    net.proxy_send("players_pos", string.sub(s, 2, -1))
end

return module
