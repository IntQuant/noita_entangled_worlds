dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local x, y = EntityGetTransform( entity_id )

local p = EntityGetParent(entity_id) or EntityGetClosestWithTag(x, y, "ew_peer")
local px,py = EntityGetTransform( p )
local d = math.abs( py - y ) + math.abs( px - x )
if ( d > 160 ) then
    EntityLoad( "data/entities/particles/teleportation_source.xml", x, y )
    EntityLoad( "data/entities/particles/teleportation_target.xml", px, py )
    EntitySetTransform( entity_id, px, py )
end