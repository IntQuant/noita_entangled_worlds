dofile_once("data/scripts/lib/utilities.lua")
dofile_once("mods/quant.ew/files/resource/shoot_projectile_fix.lua")

local entity_id    = GetUpdatedEntityID()
local pos_x, pos_y = EntityGetTransform( entity_id )

local comp = EntityGetFirstComponent(entity_id, "CharacterPlatformingComponent")

if comp == nil or ComponentGetValue2( comp, "mJetpackEmitting") < 1 then
    return
end

local vel_x, vel_y = GameGetVelocityCompVelocity(entity_id)
vel_x = -vel_x * 75
vel_y = -vel_y * 50
shoot_projectile( entity_id, "data/entities/projectiles/levitation_trail.xml", pos_x, pos_y, vel_x, vel_y )