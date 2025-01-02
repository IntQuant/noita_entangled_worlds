dofile_once("data/scripts/lib/utilities.lua")
dofile_once("mods/quant.ew/files/resource/shoot_projectile_fix.lua")

local entity_id = GetUpdatedEntityID()
local pos_x, pos_y = EntityGetTransform(entity_id)

SetRandomSeed(pos_x, pos_y)

local how_many = 16
local angle_inc = (2 * 3.14159) / how_many
local theta = math.rad(Random(1, 360))
local length = 300

local parent_id = EntityGetParent(entity_id)

if parent_id ~= NULL_ENTITY then
    for i = 1, how_many do
        local vel_x = math.cos(theta) * length
        local vel_y = math.sin(theta) * length
        theta = theta + angle_inc

        shoot_projectile(parent_id, "data/entities/misc/essences/air_bullet.xml", pos_x, pos_y, vel_x, vel_y)
    end
end
