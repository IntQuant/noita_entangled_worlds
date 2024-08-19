local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local wandfinder = dofile_once("mods/quant.ew/files/system/notplayer_ai/wandfinder.lua")

local MAX_RADIUS = 128*4

local state

local module = {}

local function log(...)
    -- GamePrint(...)
end

local function aim_at(world_x, world_y)
    local arm = EntityGetAllChildren(ctx.my_player.entity, "player_arm_r")[1]
    local ch_x, ch_y = EntityGetHotspot(arm, "hand", true)
    local dx, dy = world_x - ch_x, world_y - ch_y
    ComponentSetValue2(state.control_component, "mAimingVector", dx, dy)

    ComponentSetValue2(state.control_component, "mMousePosition", world_x, world_y)

    local dist = math.sqrt(dx * dx + dy * dy)
    if dist > 0 then
        -- ComponentSetValue2(state.control_component, "mAimingVectorNonZeroLatest", dx, dy)
        ComponentSetValue2(state.control_component, "mAimingVectorNormalized", dx/dist, dy/dist)
    end
end

local function fire_wand(enable)
    ComponentSetValue2(state.control_component, "mButtonDownFire", enable)
    ComponentSetValue2(state.control_component, "mButtonDownFire2", enable)
    if enable then
        if not state.was_firing_wand then
            ComponentSetValue2(state.control_component, "mButtonFrameFire", GameGetFrameNum()+1)
        end
        ComponentSetValue2(state.control_component, "mButtonLastFrameFire", GameGetFrameNum())
    end
    state.was_firing_wand = enable
end


local function init_state()
    state = {
        entity = ctx.my_player.entity,
        control_component = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "ControlsComponent"),
        data_component = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "CharacterDataComponent"),
        gui_component = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "InventoryGuiComponent"),


        attack_wand = wandfinder.find_attack_wand(),

        aim_up = false,
        aim_down = false,

        was_firing_wand = false,
        was_a = false,
        was_w = false,
        was_d = false,
        init_timer = 0,

        control_a = false,
        control_w = false,
        control_d = false,
    }
end

local target

local last_length

local last_did_hit = false

local function is_suitable_target(entity)
    return EntityGetIsAlive(entity) and not EntityHasTag(entity,"ew_notplayer")
end

local function choose_wand_actions()
    if state.attack_wand ~= nil and target ~= nil and EntityGetIsAlive(target) then
        local t_x, t_y = EntityGetFirstHitboxCenter(target)
        if t_x == nil then
            t_x, t_y = EntityGetTransform(target)
        end
        if state.aim_up then
            t_y = t_y - 5
        elseif state.aim_down then
            t_y = t_y + 5
        end
        aim_at(t_x, t_y)

        fire_wand(not last_did_hit and state.init_timer > 90)
        return
    end
    fire_wand(false)
end

local stop_y = false

local swap_side = false

local give_space = 100

local on_right = false

local rest = false

local function choose_movement()
    if target == nil then
        state.control_a = false
        state.control_d = false
        state.control_w = false
        stop_y = false
        swap_side = false
        on_right = false
        return
    end
    local my_x, my_y = EntityGetTransform(ctx.my_player.entity)
    local t_x, t_y = EntityGetTransform(target)
    local dist = my_x - t_x
    local LIM = give_space
    if swap_side and (on_right ~= (my_x > t_x) or GameGetFrameNum() % 300 == 299) then
        swap_side = false
    end
    if swap_side then
        LIM = 0
        give_space = 100
    end
    if dist > 0 then
        state.control_a = dist > LIM
        state.control_d = not state.control_a
    else
        state.control_d = -dist > LIM
        state.control_a = not state.control_d
    end
    if (not stop_y) and ((last_did_hit and t_y < my_y + 80) or t_y < my_y) and (GameGetFrameNum() % 300) < 200 then
        state.control_w = true
        local did_hit, _, _ = RaytracePlatforms(my_x, my_y, my_x, my_y - 5)
        if did_hit then
            state.control_w = false
            stop_y = true
        end
    else
        if stop_y and (GameGetFrameNum() % 300) > 200 then
            stop_y = false
        end
        state.control_w = (GameGetFrameNum() % 60) > 45
    end

    if last_did_hit and t_y < my_y + 40 then
        local did_hit_1, _, _ = RaytracePlatforms(my_x, my_y, t_x, my_y)
        local did_hit_2, _, _ = RaytracePlatforms(t_x, my_y, t_x, t_y)
        if did_hit_1 and (not did_hit_2) then
            swap_side = true
            on_right = my_x > t_x
        end
    end

    local did_hit_1, _, _ = RaytracePlatforms(my_x, my_y, my_x + 10, my_y)
    local did_hit_2, _, _ = RaytracePlatforms(my_x, my_y, my_x - 10, my_y)
    if (dist > 0 and dist > give_space and did_hit_2) or (dist < 0 and -dist > give_space and did_hit_1) then
        give_space = give_space + 10
        swap_side = false
    elseif give_space > 200 then
        local did_hit_3, _, _ = RaytracePlatforms(my_x, my_y, my_x + 100, my_y)
        local did_hit_4, _, _ = RaytracePlatforms(my_x, my_y, my_x - 100, my_y)
        if (dist > 0 and not did_hit_3) or (dist < 0 and not did_hit_4) then
            swap_side = true
        end
    end
    if (did_hit_1 and my_x > t_x) or (did_hit_2 and my_x < t_x) then
        swap_side = true
        on_right = my_x > t_x
    end

    if ComponentGetValue2(state.data_component, "mFlyingTimeLeft") == 0 and GameGetFrameNum() % 300 > 250 then
        rest = true
        give_space = give_space + 10
        swap_side = false
    end
    if rest and GameGetFrameNum() % 300 == 250 then
        rest = false
    end
    if rest then
        state.control_w = false
    end
end

local function position_to_area_number(x, y)
    if y < 1199 then
        return 1
    elseif y < 2735 then
        return 2
    elseif y < 4783 then
        return 3
    elseif y < 6319 then
        return 4
    elseif y < 8367 then
        return 5
    elseif y < 10415 then
        return 6
    elseif y < 12975 and (x < 2726 or x > 4135 or y < 12800) then
        return 7
    else
        return 8
    end
end

local function teleport_to_area(area)
    async(function()
        if area == 1 then
            EntitySetTransform(ctx.my_player.entity, 191, 1514)
            wait(30)
            EntitySetTransform(ctx.my_player.entity, 191, 1514)
        elseif area == 2 then
            EntitySetTransform(ctx.my_player.entity, 191, 3066)
            wait(30)
            EntitySetTransform(ctx.my_player.entity, 191, 3066)
        elseif area == 3 then
            EntitySetTransform(ctx.my_player.entity, 191, 5114)
            wait(30)
            EntitySetTransform(ctx.my_player.entity, 191, 5114)
        elseif area == 4 then
            EntitySetTransform(ctx.my_player.entity, 191, 6634)
            wait(30)
            EntitySetTransform(ctx.my_player.entity, 191, 6634)
        elseif area == 5 then
            EntitySetTransform(ctx.my_player.entity, 191, 8696)
            wait(30)
            EntitySetTransform(ctx.my_player.entity, 191, 8696)
        elseif area == 6 then
            EntitySetTransform(ctx.my_player.entity, 191, 10730)
            wait(30)
            EntitySetTransform(ctx.my_player.entity, 191, 10730)
        elseif area == 7 then
            EntitySetTransform(ctx.my_player.entity, 3244, 13084)
            wait(30)
            EntitySetTransform(ctx.my_player.entity, 3244, 13084)
        end
    end)
end

local function teleport_to_next_hm()
    --BiomeMapGetName()
    --BIOME_MAP
    --MagicNumbersGetValue

    -- main x area -5646 < x < 5120
    -- main y area -1400 < y < 14336

    -- 1st area, y < 1104, exit  191,  1514
    -- 2nd area, y < 2640, exit  191,  3066
    -- 3rd area, y < 4688, exit  191,  5114
    -- 4th area, y < 6224, exit  191,  6634
    -- 5th area, y < 8272, exit  191,  8696
    -- 6th area, y < 10320, exit 191,  10730
    -- 7th area, y < 12880, exit 3244, 13084

    local my_area_num = -1
    local others_area_num = 9
    for peer_id, player_data in pairs(ctx.players) do
        local player = player_data.entity
        local x, y = EntityGetTransform(player)
        if not (-5646 < x and x < 5120 and -1400 < y and y < 14336) then
            return
        end
        if peer_id == ctx.my_id then
            my_area_num = position_to_area_number(x, y)
        elseif is_suitable_target(player) then
            local area_num = position_to_area_number(x, y)
            if area_num < others_area_num then
                others_area_num = area_num
            end
        end
    end
    if my_area_num < others_area_num then
        teleport_to_area(others_area_num - 1)
    end
end

local camera_player = -1

local camera_target = nil

local inventory_target = nil

local function set_camera_pos()
    if camera_player < 1 then
        camera_player = 1000
    end
    local i = 0
    local cam_target = nil
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
        local t_x, t_y = EntityGetFirstHitboxCenter(cam_target)
        if t_x == nil then
            t_x, t_y = EntityGetTransform(cam_target)
        end
        GameSetCameraPos(t_x, t_y)
        if camera_target == nil then
            camera_target = ctx.my_player.entity
        end
        if camera_target ~= cam_target then
            if ctx.my_player.entity ~= camera_target then
                EntityRemoveComponent(camera_target, inventory_target)
            end

            if ctx.my_player.entity ~= cam_target then
                inventory_target = EntityAddComponent2(cam_target, "InventoryGuiComponent")
            end
            EntityRemoveComponent(camera_target, EntityGetFirstComponent(camera_target, "AudioListenerComponent"))
            EntityAddComponent2(cam_target, "AudioListenerComponent")
        end
        camera_target = cam_target
    end
end

local function better_player(length, did_hit, potential_target)
    return (last_length == nil or (not did_hit and (last_length > length or not last_did_hit))) and not IsInvisible(potential_target)
end

local left_held = false

local right_held = false

local function update()
    -- No taking control back, even after pressing esc.
    np.SetActiveHeldEntity(state.entity, state.attack_wand, false, false)
    ComponentSetValue2(state.control_component, "enabled", false)

    state.init_timer = state.init_timer + 1

    ComponentSetValue2(state.gui_component, "mActive", false)
    if inventory_target ~= nil then
        ComponentSetValue2(inventory_target, "mActive", false)
    end

    local ch_x, ch_y = EntityGetTransform(state.entity)
    local potential_targets = EntityGetInRadiusWithTag(ch_x, ch_y, MAX_RADIUS, "ew_client") or {}
    local arm = EntityGetAllChildren(ctx.my_player.entity, "player_arm_r")[1]
    local x, y = EntityGetHotspot(arm, "hand", true)

    if target ~= nil and not is_suitable_target(target) then
        target = nil
        last_length = nil
        last_did_hit = false
    end
    if target ~= nil then
        local t_x, t_y = EntityGetFirstHitboxCenter(target)
        if t_x == nil then
            t_x, t_y = EntityGetTransform(target)
        end
        state.aim_up = false
        state.aim_down = false
        local did_hit, _, _ = RaytracePlatforms(x, y, t_x, t_y)
        if did_hit then
            did_hit, _, _ = RaytracePlatforms(x, y, t_x, t_y - 5)
            if not did_hit then
                state.aim_up = true
            else
                did_hit, _, _ = RaytracePlatforms(x, y, t_x, t_y + 5)
                if not did_hit then
                    state.aim_down = true
                end
            end
        end
        last_did_hit = did_hit
    end

    log("Trying to choose target")
    for _, potential_target in ipairs(potential_targets) do
        log("Trying "..potential_target)
        if is_suitable_target(potential_target) then
            local t_x, t_y = EntityGetFirstHitboxCenter(potential_target)
            if t_x == nil then
                t_x, t_y = EntityGetTransform(potential_target)
            end
            local dx = x - t_x
            local dy = y - t_y
            local length = dx * dx + dy * dy
            local did_hit, _, _ = RaytracePlatforms(x, y, t_x, t_y)
            if better_player(length, did_hit, potential_target) then
                last_length = length
                last_did_hit = did_hit
                target = potential_target
                state.aim_up = false
                state.aim_down = false
            elseif did_hit then
                local did_hit_up, _, _ = RaytracePlatforms(x, y, t_x, t_y - 5)
                if better_player(length, did_hit_up, potential_target) then
                    last_length = length
                    last_did_hit = did_hit
                    target = potential_target
                    state.aim_up = true
                    state.aim_down = false
                elseif did_hit_up then
                    local did_hit_down, _, _ = RaytracePlatforms(x, y, t_x, t_y + 5)
                    if better_player(length, did_hit_down, potential_target) then
                        last_length = length
                        last_did_hit = did_hit
                        target = potential_target
                        state.aim_up = false
                        state.aim_down = true
                    end
                end
            end
        end
    end
    local do_kick = last_length ~= nil and last_length < 100

    if do_kick then
        fire_wand(false)
        ComponentSetValue2(state.control_component, "mButtonDownKick", true)
        ComponentSetValue2(state.control_component, "mButtonFrameKick", GameGetFrameNum()+1)
    else
        ComponentSetValue2(state.control_component, "mButtonDownKick", false)
        choose_wand_actions()
    end
    choose_movement()

    ComponentSetValue2(state.control_component, "mButtonDownLeft", state.control_a)
    if state.control_a and not state.was_a then
        ComponentSetValue2(state.control_component, "mButtonFrameLeft", GameGetFrameNum()+1)
    end
    state.was_a = state.control_a

    ComponentSetValue2(state.control_component, "mButtonDownRight", state.control_d)
    if state.control_d and not state.was_d then
        ComponentSetValue2(state.control_component, "mButtonFrameRight", GameGetFrameNum()+1)
    end
    state.was_d = state.control_d

    ComponentSetValue2(state.control_component, "mButtonDownDown", false)
    ComponentSetValue2(state.control_component, "mButtonDownUp", true)
    ComponentSetValue2(state.control_component, "mButtonDownFly", state.control_w)
    if state.control_w and not state.was_w then
        ComponentSetValue2(state.control_component, "mButtonFrameUp", GameGetFrameNum()+1)
        ComponentSetValue2(state.control_component, "mButtonFrameFly", GameGetFrameNum()+1)
    end
    state.was_w = state.control_w
    local _, y_n = EntityGetTransform(ctx.my_player.entity)
    ComponentSetValue2(state.control_component, "mFlyingTargetY", y_n - 10)

    if (GameGetFrameNum() % 300) == 299 then
        teleport_to_next_hm()
    end

    if InputIsMouseButtonJustDown(1) then
        if not left_held then
            if camera_player == -1 then
                camera_player = 1001
            end
            camera_player = camera_player - 1
            left_held = true
        end
    elseif InputIsMouseButtonJustDown(2) then
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

function module.on_world_update()
    local active = GameHasFlagRun("ew_flag_notplayer_active")
    if active then
        if state == nil then
            init_state()
        end
        update()
    else
        state = nil
    end
end

return module