local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local wandfinder = dofile_once("mods/quant.ew/files/system/notplayer_ai/wandfinder.lua")

local MAX_RADIUS = 128*3

local state = nil

local module = {}

local function log(...)
    -- GamePrint(...)
end

local function aim_at(world_x, world_y)
    local ch_x, ch_y = EntityGetTransform(state.entity)
    local dx, dy = world_x - ch_x, world_y - ch_y
    ComponentSetValue2(state.control_component, "mAimingVector", dx, dy)
    
    -- No idea what coordinate system that field uses.
    -- Writing a big positive/negative value to turn to the right side seems to work.
    local mouse_x
    if dx > 0 then
        mouse_x = 100000
    else
        mouse_x = -100000
    end
    ComponentSetValue2(state.control_component, "mMousePosition", mouse_x, 0)
    
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
        
        attack_wand = wandfinder.find_attack_wand(),

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

local target = nil

local last_length = nil

local function is_suitable_target(entity)
    local tags = EntityGetTags(entity)
    return tags ~= nil and not string.match(tags,"notplayer")
end

local function choose_wand_actions()
    if state.attack_wand ~= nil and target ~= nil then
        np.SetActiveHeldEntity(state.entity, state.attack_wand, false, false)
        local x, y = EntityGetTransform(ctx.my_player.entity)
        local t_x, t_y = EntityGetFirstHitboxCenter(target)
        if t_x == nil then
            t_x, t_y = EntityGetTransform(target)
        end
        aim_at(t_x, t_y)
        local did_hit, _, _ = RaytracePlatforms(x, y, t_x, t_y)

        fire_wand(not did_hit)
    end
end

local function choose_movement()
    state.control_w = (GameGetFrameNum() % 60) > 45
    if target == nil then
        state.control_a = false
        state.control_d = false
        return
    end
    local my_x, _ = EntityGetTransform(ctx.my_player.entity)
    local t_x, t_y = EntityGetTransform(target)
    local dist = my_x - t_x
    local LIM = 100
    if dist > 0 then
        state.control_a = dist > LIM
        state.control_d = not state.control_a
    else
        state.control_a = dist > -LIM
        state.control_d = not state.control_a
    end
end

local function update()
    -- No taking control back, even after pressing esc.
    ComponentSetValue2(state.control_component, "enabled", false)

    state.init_timer = state.init_timer + 1

    if target ~= nil and not is_suitable_target(target) then
        target = nil
        last_length = nil
    end

    log("Trying to choose target")
    local ch_x, ch_y = EntityGetTransform(state.entity)
    local potential_targets = EntityGetInRadiusWithTag(ch_x, ch_y, MAX_RADIUS, "ew_client") or {}
    local x, y = EntityGetTransform(ctx.my_player.entity)
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
            if last_length == nil or (not did_hit and last_length > length) then
                last_length = length
                target = potential_target
            end
        end
    end

    local do_kick = last_length ~= nil and last_length < 100

    if do_kick then
        fire_wand(false)
        ComponentSetValue2(state.control_component, "mButtonDownKick", true)
        ComponentSetValue2(state.control_component, "mButtonFrameKick", GameGetFrameNum()+1)
    end
    if not do_kick then
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

    ComponentSetValue2(state.control_component, "mButtonDownUp", state.control_w)
    ComponentSetValue2(state.control_component, "mButtonDownFly", state.control_w)
    if state.control_w and not state.was_w then
        ComponentSetValue2(state.control_component, "mButtonFrameUp", GameGetFrameNum()+1)
        ComponentSetValue2(state.control_component, "mButtonFrameFly", GameGetFrameNum()+1)
    end
    state.was_w = state.control_w
    local _, y = EntityGetTransform(ctx.my_player.entity)
    ComponentSetValue2(state.control_component, "mFlyingTargetY", y - 10)
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
