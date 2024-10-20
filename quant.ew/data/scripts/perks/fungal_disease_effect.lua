dofile_once("data/scripts/lib/utilities.lua")
dofile_once("data/scripts/gun/procedural/gun_action_utils.lua")
dofile_once("mods/quant.ew/files/resource/shoot_projectile_fix.lua")

local entity_id = GetUpdatedEntityID()
local root_id = EntityGetRootEntity( entity_id )
local x,y = EntityGetTransform( entity_id )
SetRandomSeed( x * entity_id, y + GameGetFrameNum() )

local rnd = Random( 1, 10 )
local vel_x = Random( -100, 100 )
local vel_y = Random( -100, 100 )

if ( rnd == 10 ) then
    shoot_projectile( root_id, "data/entities/misc/perks/spore_pod.xml", x, y, vel_x, vel_y )
elseif ( rnd > 6 ) then
    for i=1,5 do
        vel_x = Random( -400, 400 )
        vel_y = Random( -400, 100 )
        shoot_projectile( root_id, "data/entities/misc/perks/spore_pod_spike.xml", x, y, vel_x, vel_y )
    end
elseif ( rnd > 3 ) then
    for i=1,2 do
        vel_x = Random( -400, 400 )
        vel_y = Random( -400, 100 )
        shoot_projectile( root_id, "data/entities/misc/perks/spore_pod_spike.xml", x, y, vel_x, vel_y )
    end
end