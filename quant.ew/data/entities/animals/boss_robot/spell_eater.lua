dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local boss_id = EntityGetRootEntity( entity_id )
local x, y = EntityGetTransform( boss_id )
local distance_full = 36
local ax,ay = 0

local projectiles = EntityGetInRadiusWithTag( x, y, distance_full, "projectile" )

local varcomps = EntityGetComponent( boss_id, "VariableStorageComponent" )
local players = EntityGetWithTag("ew_peer")
local closest
local player_id
for _, player in ipairs(players) do
    if not EntityHasTag(player, "ew_notplayer") then
        local px, py = EntityGetTransform(player)
        local r = px * px + py * py
        if closest == nil or r < closest then
            closest = r
            player_id = player
        end
    end
end

local state = 0

if ( varcomps ~= nil ) then
    for i,v in ipairs( varcomps ) do
        local n = ComponentGetValue2( v, "name" )
        if ( n == "spell_eater" ) then
            state = ComponentGetValue2( v, "value_int" )
            break
        end
    end
end

if ( state == 0 ) then
    EntitySetComponentsWithTagEnabled( entity_id, "boss_robot_spell_eater", false )
else
    EntitySetComponentsWithTagEnabled( entity_id, "boss_robot_spell_eater", true )
    if ( player_id ~= nil ) then
        local plx,ply = EntityGetTransform( player_id )
        ax,ay = x - plx, y - ply
        local a = math.pi - math.atan2( ay, ax )

        EntitySetTransform( entity_id, x, y, 0 - a )

        if ( #projectiles > 0 ) then
            for i,projectile_id in ipairs(projectiles) do
                local px, py = EntityGetTransform( projectile_id )

                local distance = get_distance( px, py, x, y )
                local direction = get_direction( px, py, x, y )

                local dirdelta = get_direction_difference( direction, a )
                local dirdelta_deg = math.abs( math.deg( dirdelta ) )

                if ( distance < distance_full ) and ( dirdelta_deg < 82.0 ) then
                    local pcomp = EntityGetFirstComponent( projectile_id, "ProjectileComponent" )
                    local valid = true

                    if ( pcomp ~= nil ) then
                        local whoshot = ComponentGetValue2( pcomp, "mWhoShot" )

                        if ( whoshot == boss_id ) then
                            valid = false
                        else
                            ComponentSetValue2( pcomp, "on_death_explode", false )
                            ComponentSetValue2( pcomp, "on_lifetime_out_explode", false )
                        end
                    end

                    if valid then
                        EntityKill( projectile_id )
                    end
                end
            end
        end
    end
end