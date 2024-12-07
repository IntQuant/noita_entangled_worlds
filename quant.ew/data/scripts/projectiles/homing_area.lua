dofile_once("data/scripts/lib/utilities.lua")

local entity_id    = GetUpdatedEntityID()
local root_id = EntityGetRootEntity( entity_id )
local x, y = EntityGetTransform( entity_id )
local radius = 32
local targets = EntityGetInRadiusWithTag( x, y, radius, "homing_target" )
local comp = EntityGetFirstComponent( root_id, "ProjectileComponent" )

if ( comp ~= nil ) then
    local target = ComponentGetValue2( comp, "mWhoShot" )

    for i,v in ipairs( targets ) do
        if ( v ~= target ) and ( GameGetGameEffect( v, "CHARM" ) == 0 )
                and EntityGetComponent(target, "GenomeDataComponent") ~= nil
                and EntityGetComponent(v, "GenomeDataComponent") ~= nil
                and ( EntityGetHerdRelation( target, v ) < 60 ) then
            local tx, ty = EntityGetFirstHitboxCenter( v )

            EntitySetTransform( root_id, tx, ty )
            EntityApplyTransform( root_id, tx, ty )
            break
        end
    end
end