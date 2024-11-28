dofile_once( "data/scripts/lib/utilities.lua" )

local entity_id    = GetUpdatedEntityID()
local x, y = EntityGetTransform( entity_id )

local targets = EntityGetInRadiusWithTag( x, y, 20, "small_friend" )
if ( #targets > 0 ) then
    for i,t in ipairs( targets ) do
        if ( EntityHasTag( t, "polymorphed") == false ) and not EntityHasTag(t, "racing_cart") then
            EntityKill( t )
            
            EntitySetComponentsWithTagEnabled( entity_id, "driver", true )
            EntitySetComponentsWithTagEnabled( entity_id, "driverless", false )
            
            EntityAddTag( entity_id, "small_friend" )
            return
        end
    end
end
