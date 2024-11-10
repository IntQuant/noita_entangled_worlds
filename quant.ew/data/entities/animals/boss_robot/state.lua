dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local x, y = EntityGetTransform( entity_id )

local varcomps = EntityGetComponent( entity_id, "VariableStorageComponent" )
local varcomp
local state = 0
local eatercomp

if ( varcomps ~= nil ) then
    for i,v in ipairs( varcomps ) do
        local n = ComponentGetValue2( v, "name" )
        if ( n == "state" ) then
            state = ComponentGetValue2( v, "value_int" )
            varcomp = v
        elseif ( n == "spell_eater" ) then
            eatercomp = v
        end
    end
end

SetRandomSeed( x + GameGetFrameNum(), y + entity_id )

if ( varcomp ~= nil ) and ( eatercomp ~= nil ) then
    state = state + 1

    local lcomps = EntityGetComponent( entity_id, "LaserEmitterComponent" )

    if ( state == 2 ) then
        ComponentSetValue2( eatercomp, "value_int", 0 )
        local players = EntityGetInRadiusWithTag( x, y, 300, "ew_peer" )

        if ( #players > 0 ) then
            for i=1,10 do
                local a = 3.1415 * ( Random( 0, 100 ) * 0.01 )
                local length = Random( 100, 250 )
                local vx = math.cos( a ) * length
                local vy = 0 - math.sin( a ) * length

                shoot_projectile( entity_id, "data/entities/animals/boss_robot/rocket_roll.xml", x, y, vx, vy )
            end
        end
    elseif ( state == 4 ) then
        ComponentSetValue2( eatercomp, "value_int", 1 )
    elseif ( state == 6 ) then
        ComponentSetValue2( eatercomp, "value_int", 0 )

        if ( lcomps ~= nil ) then
            for k,lcomp in ipairs( lcomps ) do
                local players = EntityGetWithTag("ew_peer")
                local closest
                local p
                for _, player in ipairs(players) do
                    if not EntityHasTag(player, "ew_notplayer") then
                        local px, py = EntityGetTransform(player)
                        local r = px * px + py * py
                        if closest == nil or r < closest then
                            closest = r
                            p = player
                        end
                    end
                end

                if ( p ~= nil ) then
                    local px,py = EntityGetTransform( p )

                    local a = math.atan2( py - y, px - x )
                    ComponentSetValue2( lcomp, "laser_angle_add_rad", a )
                    ComponentObjectSetValue2( lcomp, "laser", "beam_radius", 1.5 )
                    ComponentObjectSetValue2( lcomp, "laser", "damage_to_entities", 0 )
                    ComponentObjectSetValue2( lcomp, "laser", "damage_to_cells", 10 )
                    ComponentObjectSetValue2( lcomp, "laser", "max_cell_durability_to_destroy", 2 )
                    ComponentObjectSetValue2( lcomp, "laser", "audio_enabled", false )
                    ComponentSetValue2( lcomp, "is_emitting", true )
                end
            end

        end
    elseif ( state == 8 ) then
        if ( lcomps ~= nil ) then
            for a,lcomp in ipairs( lcomps ) do
                ComponentObjectSetValue2( lcomp, "laser", "beam_radius", 10.5 )
                ComponentObjectSetValue2( lcomp, "laser", "damage_to_entities", 0.6 )
                ComponentObjectSetValue2( lcomp, "laser", "damage_to_cells", 700000 )
                ComponentObjectSetValue2( lcomp, "laser", "max_cell_durability_to_destroy", 14 )

                if ( a == 1 ) then
                    ComponentObjectSetValue2( lcomp, "laser", "audio_enabled", true )
                end
            end
        end
    elseif ( state == 10 ) then
        if ( lcomps ~= nil ) then
            for a,lcomp in ipairs( lcomps ) do
                ComponentSetValue2( lcomp, "is_emitting", false )
                ComponentObjectSetValue2( lcomp, "laser", "beam_radius", 1.5 )
                ComponentObjectSetValue2( lcomp, "laser", "damage_to_entities", 0 )
                ComponentObjectSetValue2( lcomp, "laser", "damage_to_cells", 10 )
                ComponentObjectSetValue2( lcomp, "laser", "max_cell_durability_to_destroy", 2 )
                ComponentObjectSetValue2( lcomp, "laser", "audio_enabled", false )
            end
        end
        ComponentSetValue2( eatercomp, "value_int", 1 )
    elseif ( state == 13 ) then
        local h = EntityGetWithTag( "healer" )
        if ( #h < 3 ) then
            EntityLoad( "data/entities/animals/robobase/healerdrone_physics.xml", x, y )
        end
        state = 0
    end

    ComponentSetValue2( varcomp, "value_int", state )
end