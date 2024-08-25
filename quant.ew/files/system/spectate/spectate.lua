local spectate = {}

local camera_player = -1

local camera_target

local inventory_target

local function set_camera_pos()
    if camera_player < 1 and camera_player ~= -1 then
        camera_player = 1000
    end
    local i = 0
    local cam_target
    for peer_id, potential_target in pairs(ctx.players) do
        local entity = potential_target.entity
        i = i + 1
        if i == camera_player or (i == -1 and peer_id == ctx.my_id) then
            cam_target = entity
            camera_player = i
        end
    end
    if camera_player == 1000 then
        camera_player = i
        set_camera_pos()
    elseif i ~= 0 and i < camera_player then
        camera_player = 1
        set_camera_pos()
    elseif cam_target ~= nil then
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
end

local left_held = false

local right_held = false

function spectate.on_world_update()
    if InputIsKeyDown(54) then
        if not left_held then
            if camera_player == -1 then
                camera_player = 1001
            end
            camera_player = camera_player - 1
            left_held = true
        end
    elseif InputIsKeyDown(55) then
        if not right_held then
            if camera_player == -1 then
                camera_player = 0
            end
            camera_player = camera_player + 1
            right_held = true
        end
    else
        left_held = false
        right_held = false
    end
    set_camera_pos()
end

return spectate