dofile_once("data/scripts/lib/utilities.lua")

local lerp_amount = 0.975
local bob_h = 6
local bob_w = 20
local bob_speed_y = 0.065
local bob_speed_x = 0.01421

local entity_id = GetUpdatedEntityID()
local pos_x, pos_y = EntityGetTransform(entity_id)

if pos_x == 0 and pos_y == 0 then
    -- get position from wand when starting
    pos_x, pos_y = EntityGetTransform(EntityGetParent(entity_id))
end

-- ghost continously lerps towards a target that floats around the parent
local target_x, target_y = EntityGetTransform(EntityGetParent(entity_id))
if target_x == nil then
    return
end
target_y = target_y - 10

local time = CrossCall("ew_host_frame_num")
local r
local var_rnd = EntityGetFirstComponentIncludingDisabled(entity_id, "VariableStorageComponent", "ew_ghost_rnd")
if var_rnd == nil then
    r = ProceduralRandomf(entity_id, 0, -1, 1)
    EntityAddComponent2(entity_id, "VariableStorageComponent", { _tags = "ew_ghost_rnd", value_float = r })
else
    r = ComponentGetValue(var_rnd, "value_float")
end

-- randomize times and speeds slightly so that multiple ghosts don't fly identically
time = time + r * 10000
bob_speed_y = bob_speed_y + (r * bob_speed_y * 0.1)
bob_speed_x = bob_speed_x + (r * bob_speed_x * 0.1)
lerp_amount = lerp_amount - (r * lerp_amount * 0.01)

-- bob
target_y = target_y + math.sin(time * bob_speed_y) * bob_h
target_x = target_x + math.sin(time * bob_speed_x) * bob_w

local dist_x = pos_x - target_x

-- move towards target
pos_x, pos_y = vec_lerp(pos_x, pos_y, target_x, target_y, lerp_amount)
EntitySetTransform(entity_id, pos_x, pos_y, 0, 1, 1)

-- animation state
edit_component(entity_id, "SpriteComponent", function(comp, vars)
    local current_anim = ComponentGetValue(comp, "rect_animation")

    -- float when nearby and fly when further away
    local mode = "float_"
    if math.abs(dist_x) > 28 then
        mode = "fly_"
    end

    -- check if changing the animation is needed based on current animation and heading
    if dist_x < 2 and current_anim ~= mode .. "right" then
        ComponentSetValue(comp, "rect_animation", mode .. "right")
    elseif dist_x > 2 and current_anim ~= mode .. "left" then
        ComponentSetValue(comp, "rect_animation", mode .. "left")
    end
end)
