local spectate = {}

local camera_player

local re_cam = false

local camera_player_id

local camera_target

local cam_target

local inventory_target

local left_held = false

local right_held = false

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
    if cam_target == ctx.my_player.entity and not EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
        GameSetCameraFree(false)
        if camera_target == nil then
            camera_target = ctx.my_player.entity
        elseif camera_target ~= ctx.my_player.entity then
            EntityAddComponent2(cam_target, "AudioListenerComponent")
            local audio = EntityGetFirstComponent(camera_target, "AudioListenerComponent")
            local keep_alive = EntityGetFirstComponent(camera_target, "StreamingKeepAliveComponent")
            EntityRemoveComponent(camera_target, audio)
            EntityRemoveComponent(camera_target, keep_alive)
            EntityRemoveComponent(camera_target, inventory_target)
            camera_target = cam_target
        end
        return
    end
    GameSetCameraFree(true)
    local t_x, t_y = EntityGetFirstHitboxCenter(cam_target)
    if t_x == nil then
        t_x, t_y = EntityGetTransform(cam_target)
    end
    GameSetCameraPos(t_x, t_y)
    if camera_target == nil then
        camera_target = ctx.my_player.entity
    end
    if camera_target ~= cam_target then
        if ctx.my_player.entity ~= camera_target and inventory_target ~= nil then
            EntityRemoveComponent(camera_target, inventory_target)
        end

        inventory_target = nil
        if ctx.my_player.entity ~= cam_target then
            inventory_target = EntityAddComponent2(cam_target, "InventoryGuiComponent")
        end
        local audio = EntityGetFirstComponent(camera_target, "AudioListenerComponent")
        local keep_alive = EntityGetFirstComponent(camera_target, "StreamingKeepAliveComponent")
        if audio ~= nil then
            EntityRemoveComponent(camera_target, audio)
            if camera_target ~= ctx.my_player.entity then
                EntityRemoveComponent(camera_target, keep_alive)
            end
            EntityAddComponent2(cam_target, "AudioListenerComponent")
            if cam_target ~= ctx.my_player.entity then
                EntityAddComponent2(cam_target, "StreamingKeepAliveComponent")
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
                cam_target = potential_target.entity
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
    if camera_player == nil then
        camera_player = get_me()
        camera_player_id = ctx.my_id
    end
    if last_len == nil then
        last_len = number_of_players()
    end
    if last_len ~= number_of_players() or (cam_target ~= nil and not EntityGetIsAlive(cam_target)) then
        update_i()
        last_len = number_of_players()
    end

    if InputIsKeyDown(54) then
        if not left_held then
            camera_player = camera_player - 1
            if camera_player < 1 then
                camera_player = last_len
            end
            left_held = true
            re_cam = true
        end
    elseif InputIsKeyDown(55) then
        if not right_held then
            camera_player = camera_player + 1
            if camera_player > last_len then
                camera_player = 1
            end
            right_held = true
            re_cam = true
        end
    else
        left_held = false
        right_held = false
    end
    set_camera_pos()
end

return spectate