function get_herd_id( entity_id )
    local genome = EntityGetFirstComponentIncludingDisabled(entity_id, "GenomeDataComponent")
    return ComponentGetValue2(genome, "herd_id")
end

function shoot_projectile( who_shot, entity_file, x, y, vel_x, vel_y, send_message )
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