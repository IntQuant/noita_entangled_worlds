dofile_once("data/scripts/lib/utilities.lua")

local range_near = 40 -- if below, stop chase
local range_far = 250
local range_max = 600 -- if above, stop chase

local speed_near = 0.5 -- speed used at range_near
local speed_far = 1 -- speed used at range_far

local entity_id = GetUpdatedEntityID()
local pos_x, pos_y = EntityGetTransform(entity_id)

local function set_fly_speed(spd)
    component_write( EntityGetFirstComponent( entity_id, "CharacterPlatformingComponent" ), { fly_speed_mult = spd, fly_velocity_x = spd * 2 } )
end

local target = EntityGetClosestWithTag(pos_x, pos_y, "ew_peer")
if not target or target == 0 then
    set_fly_speed(40)
    return
end

local vx, vy = EntityGetTransform(target)
vx, vy = vec_sub(vx, vy, pos_x, pos_y)
local dist = get_magnitude(vx, vy, pos_x, pos_y)

if dist < range_near or dist > range_max then
    set_fly_speed(40)
    return
end

-- move directly towards player
set_fly_speed(0)
local speed = map(dist, range_near, range_far, speed_near, speed_far)
speed = clamp(speed, speed_near, speed_far)
vx, vy = vec_normalize(vx, vy)
vx, vy = vec_mult(vx, vy, speed)

pos_x, pos_y = vec_add(pos_x, pos_y, vx, vy)
EntitySetTransform(entity_id, pos_x, pos_y)