dofile_once("data/scripts/lib/utilities.lua")

local entity_id    = GetUpdatedEntityID()
local x, y = EntityGetTransform( entity_id )

local parent_id = EntityGetParent( entity_id )

local target_id = 0

if ( parent_id ~= NULL_ENTITY ) then
    local comp = EntityGetFirstComponent( parent_id, "GenomeDataComponent" )

    if ( comp == nil ) then
        target_id = parent_id
    else
        target_id = entity_id
    end
else
    target_id = entity_id
end

if ( target_id ~= NULL_ENTITY ) then
    local rndv = 0
    for _, var in ipairs(EntityGetComponent(entity_id, "VariableStorageComponent") or {}) do
        if ComponentGetValue(var, "name") == "ew_transmutation" then
            rndv = ComponentGetValue(var, "value_int")
        end
    end
    SetRandomSeed( rndv + 423, rndv - 385 )

    local fizzle = Random( 1, 10 )

    local vel_x,vel_y = 0,0

    edit_component( target_id, "VelocityComponent", function(comp,vars)
        vel_x,vel_y = ComponentGetValueVector2( comp, "mVelocity" )
    end)

    if ( fizzle == 1 ) then
        for i=1,5 do
            shoot_projectile_from_projectile( target_id, "data/entities/projectiles/deck/fizzle.xml", x, y, vel_x, vel_y )
        end

        EntityKill( target_id )
    end
end