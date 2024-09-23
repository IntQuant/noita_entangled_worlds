local nickname = dofile_once("mods/quant.ew/files/system/nickname.lua")

local spectate = {}

local gui = GuiCreate()

local camera_player

local re_cam = false

local camera_player_id

local camera_target

local cam_target

local inventory_target

local was_notplayer = false

local has_switched = false

local function get_me()
    local i = 0
    for peer_id, _ in pairs(ctx.players) do
        i = i + 1
        if peer_id == ctx.my_id then
            return i
        end
    end
end

local function target()
    if cam_target.entity == ctx.my_player.entity and not EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
        local sprite = EntityGetFirstComponent(ctx.my_player.entity, "SpriteComponent", "ew_nickname")
        if sprite ~= nil then
            EntityRemoveComponent(ctx.my_player.entity, sprite)
        end
        GameSetCameraFree(false)
        if camera_target == nil then
            camera_target = ctx.my_player
        elseif camera_target.entity ~= ctx.my_player.entity then
            local audio = EntityGetFirstComponent(camera_target.entity, "AudioListenerComponent")
            local audio_n = EntityAddComponent2(cam_target.entity, "AudioListenerComponent")
            if audio == nil then
                ComponentSetValue2(audio_n, "z", -60)
            else
                ComponentSetValue2(audio_n, "z", ComponentGetValue2(audio, "z"))
            end
            local keep_alive = EntityGetFirstComponent(camera_target.entity, "StreamingKeepAliveComponent")
            EntityRemoveComponent(camera_target.entity, audio)
            if keep_alive ~= nil then
                EntityRemoveComponent(camera_target.entity, keep_alive)
            end
            EntityRemoveComponent(camera_target.entity, inventory_target)
            camera_target = cam_target
        end
        return
    end
    GameSetCameraFree(true)
    if not EntityGetIsAlive(cam_target.entity) then
        return
    end
    local my_x, my_y = GameGetCameraPos()
    local mx, my
    if GameGetIsGamepadConnected() then
        local controls = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "ControlsComponent")
        mx, my = ComponentGetValue2(controls, "mGamePadCursorInWorld")
    else
        mx, my = DEBUG_GetMouseWorld()
    end
    local to_x, to_y = EntityGetTransform(cam_target.entity)
    local t_x, t_y = to_x + (mx - to_x) / 3 , to_y + (my - to_y) / 3
    local dx, dy = t_x - my_x, t_y - my_y
    local di = dx * dx + dy * dy
    if di > 256 * 256 then
        GameSetCameraPos(to_x, to_y)
    else
        local cos, sin = dx / 512, dy / 512
        local d = math.sqrt(di)
        dx, dy = d * cos, d * sin
        GameSetCameraPos(my_x + dx, my_y + dy)
    end
    if camera_target == nil then
        camera_target = ctx.my_player
    end
    if camera_target.entity ~= cam_target.entity then
        if camera_target == ctx.my_player then
            nickname.add_label(ctx.my_player.entity, ctx.my_player.name, "data/fonts/font_pixel_white.xml", 0.75)
        end
        if ctx.my_player.entity ~= camera_target.entity and inventory_target ~= nil then
            EntityRemoveComponent(camera_target.entity, inventory_target)
        end

        inventory_target = nil
        if ctx.my_player.entity ~= cam_target.entity then
            inventory_target = EntityAddComponent2(cam_target.entity, "InventoryGuiComponent")
        end
        local audio = EntityGetFirstComponent(camera_target.entity, "AudioListenerComponent")
        local keep_alive = EntityGetFirstComponent(camera_target.entity, "StreamingKeepAliveComponent")
        if audio ~= nil then
            local audio_n = EntityAddComponent2(cam_target.entity, "AudioListenerComponent")
            ComponentSetValue2(audio_n, "z", ComponentGetValue2(audio, "z"))
            EntityRemoveComponent(camera_target.entity, audio)
            if camera_target.entity ~= ctx.my_player.entity and keep_alive ~= nil and not EntityHasTag(camera_target.entity, "ew_notplayer") then
                EntityRemoveComponent(camera_target.entity, keep_alive)
            end
            if cam_target.entity ~= ctx.my_player.entity and not EntityHasTag(cam_target, "ew_notplayer") then
                EntityAddComponent2(cam_target.entity, "StreamingKeepAliveComponent")
            end
        end
    end
    camera_target = cam_target
end

local function set_camera_pos()
    if cam_target == nil or re_cam then
        local i = 0
        for peer_id, potential_target in pairs(ctx.players) do
            i = i + 1
            if i == camera_player then
                cam_target = potential_target
                camera_player = i
                re_cam = false
                camera_player_id = peer_id
                break
            end
        end
        if cam_target == nil then
            camera_player = get_me()
            camera_player_id = ctx.my_id
            re_cam = true
            set_camera_pos()
        else
            target()
        end
    else
        target()
    end
end

local function update_i()
    local i = 0
    for peer_id, _ in pairs(ctx.players) do
        i = i + 1
        if peer_id == camera_player_id then
            camera_player = i
            re_cam = true
            return
        end
    end
    camera_player = get_me()
    camera_player_id = ctx.my_id
end

local function number_of_players()
    local i = 0
    for _, _ in pairs(ctx.players) do
        i = i + 1
    end
    return i
end

local last_len

function spectate.on_world_update()
    if EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
        was_notplayer = true
    elseif was_notplayer then
        was_notplayer = false
        camera_player = get_me()
        camera_player_id = ctx.my_id
        re_cam = true
    end
    if camera_player == nil then
        camera_player = get_me()
        camera_player_id = ctx.my_id
        re_cam = true
    end
    if last_len == nil then
        last_len = number_of_players()
    end
    if last_len ~= number_of_players() or (cam_target ~= nil and not EntityGetIsAlive(cam_target.entity)) then
        update_i()
        last_len = number_of_players()
    end

    if InputIsKeyJustDown(54) or InputIsJoystickButtonJustDown(0, 13) then
        camera_player = camera_player - 1
        if camera_player < 1 then
            camera_player = last_len
        end

        has_switched = true
        re_cam = true
    elseif InputIsKeyJustDown(55) or InputIsJoystickButtonJustDown(0, 14) then
        camera_player = camera_player + 1
        if camera_player > last_len then
            camera_player = 1
        end

        has_switched = true
        re_cam = true
    end
    set_camera_pos()
    ctx.spectating_over_peer_id = camera_player_id

    GuiStartFrame(gui)
    GuiZSet(gui, 11)
    if GameHasFlagRun("ew_flag_notplayer_active") and not has_switched then
        local w, h = GuiGetScreenDimensions(gui)
        local text
        if GameGetIsGamepadConnected() then
            text = "Use d-pad-left and d-pad-right keys to spectate over other players."
        else
            text = "Use ',' and '.' keys to spectate over other players."
        end
        local tw, th = GuiGetTextDimensions(gui, text)
        GuiText(gui, w-2-tw, h-1-th, text)
    end
    if cam_target.entity ~= ctx.my_player.entity then
        local inv_spec = EntityGetFirstComponent(cam_target.entity, "InventoryGuiComponent")
        local inv_me = EntityGetFirstComponent(ctx.my_player.entity, "InventoryGuiComponent")
        if inv_spec ~= nil then
            ComponentSetValue2(inv_spec, "mActive", false)
        end
        ComponentSetValue2(inv_me, "mActive", false)
    elseif GameGetFrameNum() % 10 == 0 then
        for peer_id, data in pairs(ctx.players) do
            if peer_id ~= ctx.my_id then
                local audio = EntityGetFirstComponent(data.entity, "AudioListenerComponent")
                local inv_target = EntityGetFirstComponent(data.entity, "InventoryGuiComponent")
                local keep_alive = EntityGetFirstComponent(data.entity, "StreamingKeepAliveComponent")
                if audio ~= nil then
                    EntityRemoveComponent(data.entity, audio)
                end
                if keep_alive ~= nil and not EntityHasTag(data.entity, "ew_notplayer") then
                    EntityRemoveComponent(data.entity, keep_alive)
                end
                if inv_target ~= nil then
                    EntityRemoveComponent(data.entity, inv_target)
                end
            end
        end
    end
end

return spectate