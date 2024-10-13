dofile_once("data/scripts/lib/utilities.lua")

local function get_herd_id( entity_id )
    local genome = EntityGetFirstComponentIncludingDisabled(entity_id, "GenomeDataComponent")
    local herd = ComponentGetValue2(genome, "herd_id")
    return herd
end

local function shoot_projectile( who_shot, entity_file, x, y, vel_x, vel_y, send_message )
    local entity_id = EntityLoad( entity_file, x, y )
    local herd_id   = get_herd_id( who_shot )
    if( send_message == nil ) then send_message = true end

    GameShootProjectile( who_shot, x, y, x+vel_x, y+vel_y, entity_id, send_message )

    edit_component( entity_id, "ProjectileComponent", function(comp,vars)
        vars.mWhoShot       = who_shot
        vars.mShooterHerdId = herd_id
    end)

    edit_component( entity_id, "VelocityComponent", function(comp,vars)
        ComponentSetValueVector2( comp, "mVelocity", vel_x, vel_y )
    end)

    return entity_id
end

function wand_fired( wand_id )
    local projectile_velocity = 600

    local entity_id = GetUpdatedEntityID()
    local children = EntityGetAllChildren( entity_id )
    local ghost_ids = {}
    local shooter = EntityGetRootEntity(entity_id)

    if ( children ~= nil ) then
        for i,v in ipairs( children ) do
            if EntityHasTag( v, "angry_ghost" ) then
                table.insert( ghost_ids, v )
            end
        end
    end

    if ( wand_id ~= nil ) and ( wand_id ~= NULL_ENTITY ) then
        for a,ghost_id in ipairs( ghost_ids ) do
            local pos_x, pos_y = EntityGetTransform( ghost_id )
            local comp_cd = EntityGetFirstComponent( ghost_id, "VariableStorageComponent", "angry_ghost_cooldown" )

            if ( comp_cd ~= nil ) then
                local cd = ComponentGetValue2( comp_cd, "value_int" )

                if ( cd == 0 ) then
                    SetRandomSeed(pos_x + GameGetFrameNum(), pos_y)
                    projectile_velocity = Random( 550, 750 )

                    local x,y,dir = EntityGetTransform( wand_id )
                    local comp = EntityGetFirstComponent( ghost_id, "VariableStorageComponent", "angry_ghost_projectile_memory" )
                    local projectile = "data/entities/projectiles/deck/light_bullet.xml"

                    if ( comp ~= nil ) then
                        projectile = ComponentGetValue2( comp, "value_string" )
                    end

                    if ( #projectile == 0 ) then
                        projectile = "data/entities/projectiles/deck/light_bullet.xml"
                    end

                    -- print( projectile )

                    if ( #projectile > 0 ) then
                        local vel_x = math.cos( 0 - dir ) * projectile_velocity
                        local vel_y = 0 - math.sin( 0 - dir ) * projectile_velocity
                        shoot_projectile( shooter, projectile, pos_x, pos_y, vel_x, vel_y)

                        cd = 4
                    end
                end

                ComponentSetValue2( comp_cd, "value_int", cd )
            end
        end
    end
end