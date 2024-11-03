dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local root_id = EntityGetRootEntity( entity_id )
local x, y = EntityGetTransform( entity_id )

local pcomp = 0
local scomp = 0
local timer = 0

local c = EntityGetComponent( entity_id, "VariableStorageComponent" )
if ( c ~= nil ) then
    for i,v in ipairs( c ) do
        local n = ComponentGetValue2( v, "name" )
        if ( n == "phase_timer" ) then
            timer = ComponentGetValue2( v, "value_int" )
            pcomp = v
        end
    end
end

local hcomp = EntityGetFirstComponentIncludingDisabled( root_id, "HitboxComponent" )

if ( pcomp ~= 0 ) and ( hcomp ~= nil ) then
    timer = timer + 1

    local p = EntityGetInRadiusWithTag( x, y, 160, "ew_peer" )
    local eye = EntityGetFirstComponent( entity_id, "SpriteComponent" )
    if ( eye ~= nil ) then
        local current = ComponentGetValue2( eye, "rect_animation" )

        if ( #current == 0 ) then
            ComponentSetValue2( eye, "rect_animation", "closed" )
            EntitySetComponentIsEnabled( entity_id, hcomp, false )
        end

        if ( #p == 0 ) then
            if ( current == "opened" ) then
                ComponentSetValue2( eye, "rect_animation", "close" )
                ComponentSetValue2( eye, "next_rect_animation", "closed" )
                timer = 0
                EntitySetComponentIsEnabled( entity_id, hcomp, false )
            elseif ( current == "close" ) and ( timer > 36 ) then
                ComponentSetValue2( eye, "rect_animation", "closed" )
                ComponentSetValue2( eye, "next_rect_animation", "closed" )
                timer = 0
            elseif ( current == "open" ) and ( timer > 36 ) then
                ComponentSetValue2( eye, "rect_animation", "opened" )
                ComponentSetValue2( eye, "next_rect_animation", "opened" )
                timer = 0
                EntitySetComponentIsEnabled( entity_id, hcomp, true )
            end
        else
            if ( current == "closed" ) then
                ComponentSetValue2( eye, "rect_animation", "open" )
                ComponentSetValue2( eye, "next_rect_animation", "opened" )
                timer = 0
            elseif ( current == "open" ) and ( timer > 36 ) then
                ComponentSetValue2( eye, "rect_animation", "opened" )
                ComponentSetValue2( eye, "next_rect_animation", "opened" )
                timer = 0
                EntitySetComponentIsEnabled( entity_id, hcomp, true )
            elseif ( current == "close" ) and ( timer > 36 ) then
                ComponentSetValue2( eye, "rect_animation", "closed" )
                ComponentSetValue2( eye, "next_rect_animation", "closed" )
                timer = 0
            end
        end

        if ( current == "opened" ) and ( timer > 360 ) then
            timer = 0
            SetRandomSeed( x + y, GameGetFrameNum() )
            local offset = Random( 1, 100 ) * 0.01 * math.pi
            local inc = ( math.pi * 2 ) / 8

            for a=0,7 do
                local vx = math.cos( offset + inc * a ) * 80
                local vy = 0 - math.sin( offset + inc * a ) * 80

                shoot_projectile( root_id, "data/entities/animals/boss_fish/orb_big.xml", x, y, vx, vy )
            end
        end
    end

    ComponentSetValue2( pcomp, "value_int", timer )
end