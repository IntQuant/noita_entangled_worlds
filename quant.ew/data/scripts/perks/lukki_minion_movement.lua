dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
if not EntityGetIsAlive(entity_id) then
    return
end
local x, y = EntityGetTransform( entity_id )
local px, py = x, y
local owner_id = 0

local comps = EntityGetComponent( entity_id, "VariableStorageComponent" )
local targets = EntityGetInRadiusWithTag( x, y, 160, "projectile_player" )
local targets2 = EntityGetInRadiusWithTag( x, y, 32, "enemy" )
local target = 0
local memorycomp
local swaying = true

if ( comps ~= nil ) then
    for i,v in ipairs( comps ) do
        local name = ComponentGetValue2( v, "name" )

        if ( name == "memory" ) then
            memorycomp = v
            target = ComponentGetValue2( v, "value_int" )
            local test = EntityGetTransform( target )

            if ( test == nil ) then
                target = 0
            end
        elseif ( name == "owner_id" ) then
            owner_id = ComponentGetValue2( v, "value_int" )
            px,py = EntityGetTransform( owner_id )

            if ( px == nil ) or ( py == nil ) then
                px,py = x,y
            end
        end
    end
end

local cvx, cvy = 0, 0
local physcomp = EntityGetFirstComponent( entity_id, "PhysicsBodyComponent" )
if ( physcomp ~= nil ) then
    cvx,cvy = PhysicsGetComponentVelocity( entity_id, physcomp )
end

if ( #targets > 0 ) and ( #targets2 == 0 ) and ( target == 0 ) then
    SetRandomSeed( x + entity_id, px + py )
    local rnd = Random( 1, #targets )

    target = targets[rnd]

    if ( memorycomp ~= nil ) then
        ComponentSetValue2( memorycomp, "value_int", target )
    end
end

if ( #targets2 > 0 ) then
    SetRandomSeed( x + entity_id, px + py )
    local rnd = Random( 1, #targets2 )

    target = targets2[rnd]
    swaying = false
end

if ( target ~= 0 ) then
    local test = EntityGetTransform( target )

    if ( test ~= nil ) then
        px, py = EntityGetFirstHitboxCenter( target )
    end
end

if swaying then
    local arc = GameGetFrameNum() * 0.01 + entity_id
    local length = 12

    px = px + math.cos( arc ) * length + math.sin( 0 - arc ) * length
    py = py - math.sin( arc ) * length - math.cos( 0 - arc ) * length
end

local dir = get_direction( x, y, px, py )
local dist = math.min( get_distance( x, y, px, py ), 32 )

local vel_x = 0 - (math.cos( dir ) * dist)
local vel_y = 0 - (0 - math.sin( dir ) * dist)

if ( ( x > px ) and ( cvx > 0 ) ) or ( ( x < px ) and ( cvx < 0 ) ) then
    vel_x = vel_x * 4
end

if ( ( y > py ) and ( cvy > 0 ) ) or ( ( y < py ) and ( cvy < 0 ) ) then
    vel_y = vel_y * 4
end

PhysicsApplyForce( entity_id, vel_x, vel_y )

if ( owner_id ~= 0 ) then
    x, y = EntityGetTransform( entity_id )
    local ox, oy = EntityGetTransform( owner_id )
    if ox == nil then
        return
    end
    dist = math.abs( x - ox ) + math.abs( y - oy )

    if ( dist > 900 ) then
        EntityLoad( "data/entities/particles/teleportation_source.xml", x, y )
        EntityLoad( "data/entities/particles/teleportation_target.xml", ox, oy )
        EntitySetTransform( entity_id, ox, oy )
        EntityApplyTransform( entity_id, ox, oy )
    end
end