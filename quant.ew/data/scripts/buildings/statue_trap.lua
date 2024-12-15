dofile_once("data/scripts/lib/utilities.lua")

local entity_id    = GetUpdatedEntityID()
local pos_x, pos_y = EntityGetTransform( entity_id )

local targets = EntityGetInRadiusWithTag( pos_x, pos_y, 32, "ew_peer" )

if ( targets ~= nil ) and ( #targets > 0 ) then
    EntityLoad( "data/entities/animals/statue.xml", pos_x, pos_y )
    EntityLoad( "data/entities/particles/dust_explosion.xml", pos_x, pos_y )
    EntityKill( entity_id )

    GamePlaySound( "data/audio/Desktop/animals.bank", "animals/statue/appear", pos_x, pos_y )
end