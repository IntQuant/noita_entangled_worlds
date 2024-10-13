dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local root_id = EntityGetRootEntity( entity_id )
local x, y = EntityGetTransform( entity_id )
local radius = 160

local projectiles = EntityGetInRadiusWithTag( x, y, radius, "projectile" )
local count = 0
local expcount = 0
local who_shot
local comp = EntityGetFirstComponent( entity_id, "ProjectileComponent" )
if ( comp ~= nil ) then
    who_shot = ComponentGetValue2( comp, "mWhoShot" )
end

if ( who_shot ~= nil ) and ( comp ~= nil ) then
    for i,projectile_id in ipairs(projectiles) do
        if ( projectile_id ~= root_id ) and ( projectile_id ~= entity_id ) and ( EntityHasTag( projectile_id, "spells_to_power_target" ) == false ) and ( EntityHasTag( projectile_id, "projectile_not" ) == false ) then
            local comp2 = EntityGetFirstComponent( projectile_id, "ProjectileComponent" )
            local delete = false

            if ( comp2 ~= nil ) then
                local who_shot2 = ComponentGetValue2( comp2, "mWhoShot" )

                if ( who_shot == who_shot2 ) and ( who_shot ~= NULL_ENTITY ) then
                    delete = true
                    ComponentSetValue2( comp2, "on_death_explode", false )
                    ComponentSetValue2( comp2, "on_lifetime_out_explode", false )
                    ComponentSetValue2( comp2, "collide_with_entities", false )
                    ComponentSetValue2( comp2, "collide_with_world", false )
                    ComponentSetValue2( comp2, "lifetime", 999 )
                end
            end

            if delete then
                local amount = ComponentGetValue2( comp2, "damage" ) or 0.1
                local amount2 = tonumber( ComponentObjectGetValue2( comp2, "config_explosion", "damage" ) ) or 0.1

                amount = amount * 10
                amount2 = amount2 * 10

                count = count + amount
                expcount = expcount + amount2

                local px, py = EntityGetTransform( projectile_id )
                EntityLoad("data/entities/particles/poof_red_tiny.xml", px, py)

                if delete then
                    EntityAddComponent( projectile_id, "LifetimeComponent",
                    {
                        lifetime = "1",
                    } )
                end
            end
        end
    end

    local totalcount = count + expcount

    local damage = ComponentGetValue2( comp, "damage" )
    local expdamage = ComponentObjectGetValue2( comp, "config_explosion", "damage" )
    local exprad = ComponentObjectGetValue2( comp, "config_explosion", "explosion_radius" )

    damage = damage + count * 0.1
    expdamage = expdamage + expcount * 0.1
    exprad = math.min( 120, math.floor( exprad + math.log( totalcount * 10.5 ) ) )

    ComponentSetValue2( comp, "damage", damage )
    ComponentObjectSetValue2( comp, "config_explosion", "damage", expdamage )
    ComponentObjectSetValue2( comp, "config_explosion", "explosion_radius", exprad )

    local effect_id = EntityLoad("data/entities/particles/tinyspark_red_large.xml", x, y)
    EntityAddChild( root_id, effect_id )

    edit_component( effect_id, "ParticleEmitterComponent", function(comp3,vars)
        local part_min = math.min( math.floor( totalcount * 0.5 ), 100 )
        local part_max = math.min( totalcount + 1, 120 )

        -- NOTE( Petri ) 1.3.2023 - this fixes the crash. Thanks Letaali
        if( part_min < 0 ) then part_min = 0 end
        if( part_max < 0 ) then part_max = 0 end

        ComponentSetValue2( comp3, "count_min", part_min )
        ComponentSetValue2( comp3, "count_max", part_max )
    end)
end