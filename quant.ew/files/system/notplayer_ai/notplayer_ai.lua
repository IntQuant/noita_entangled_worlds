local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")

local MAX_RADIUS = 128*3

local state = nil

local module = {}

local function log(...)
    -- GamePrint(...)
end

local function aim_at(world_x, world_y)
    local ch_x, ch_y = EntityGetTransform(state.entity)
    local dx, dy = world_x - ch_x, world_y - ch_y
    ComponentSetValue2(state.control, "mAimingVector", dx, dy)
    
    -- No idea what coordinate system that field uses.
    -- Writing a big positive/negative value to turn to the right side seems to work.
    local mouse_x
    if dx > 0 then
        mouse_x = 100000
    else
        mouse_x = -100000
    end
    ComponentSetValue2(state.control, "mMousePosition", mouse_x, 0)
    
    local dist = math.sqrt(dx * dx + dy * dy)
    if dist > 0 then
        -- ComponentSetValue2(state.control, "mAimingVectorNonZeroLatest", dx, dy)
        ComponentSetValue2(state.control, "mAimingVectorNormalized", dx/dist, dy/dist)
    end
end


local function init_state()
    state = {
        entity = ctx.my_player.entity,
        control = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "ControlsComponent")
    }
    ComponentSetValue2(state.control, "enabled", false)
end

local target = nil

local function is_suitable_target(entity)
    if not EntityGetIsAlive(entity) then
        return false
    end
    return true
end

local function choose_wand_actions()
    if target ~= nil then
        local t_x, t_y = EntityGetTransform(target)
        aim_at(t_x, t_y)
    end
end

local function choose_movement()

end

local function update()
    if target ~= nil and not is_suitable_target(target) then
        target = nil
    end

    if target == nil then
        log("Trying to choose target")
        local ch_x, ch_y = EntityGetTransform(state.entity)
        local potential_targets = EntityGetInRadiusWithTag(ch_x, ch_y, MAX_RADIUS, "ew_client") or {}
        for _, potential_target in ipairs(potential_targets) do
            log("Trying "..potential_target)
            if is_suitable_target(potential_target) then
                target = potential_target
                break
            end
        end
    end

    choose_wand_actions()
    choose_movement()
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