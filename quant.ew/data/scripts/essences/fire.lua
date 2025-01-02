dofile_once("data/scripts/lib/utilities.lua")
dofile_once("mods/quant.ew/files/resource/shoot_projectile_fix.lua")

local entity_id = GetUpdatedEntityID()
local pos_x, pos_y = EntityGetTransform(entity_id)

local parent_id = EntityGetParent(entity_id)

if parent_id ~= NULL_ENTITY then
    shoot_projectile(parent_id, "data/entities/misc/essences/fire_explosion.xml", pos_x, pos_y, 0, 0)
end
