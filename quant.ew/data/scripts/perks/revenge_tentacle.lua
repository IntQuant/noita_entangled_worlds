dofile_once("data/scripts/lib/utilities.lua")
dofile_once("mods/quant.ew/files/resource/shoot_projectile_fix.lua")

function damage_received( damage, desc, entity_who_caused, is_fatal )
    local entity_id    = GetUpdatedEntityID()
    local x, y = EntityGetTransform( entity_id )

    -- don't revenge tentacle on heal
    if( damage < 0 ) then return end

    SetRandomSeed( GameGetFrameNum(), x + y + entity_id )
    if entity_who_caused == 0 or entity_who_caused == 1 or ( entity_who_caused == entity_id ) or ( ( EntityGetParent( entity_id ) ~= NULL_ENTITY ) and ( entity_who_caused == EntityGetParent( entity_id ) ) ) then return end

    -- check that we're only shooting every 10 frames
    if script_wait_frames( entity_id, 2 ) then  return  end

    local angle = math.rad( Random( 1, 360 ) )
    local angle_random = math.rad( Random( -5, 5 ) )
    local vel_x = 0
    local vel_y = 0
    local length = 900

    if ( entity_who_caused ~= nil ) and ( entity_who_caused ~= NULL_ENTITY ) then
        local ex, ey = EntityGetTransform( entity_who_caused )

        if ( ex ~= nil ) and ( ey ~= nil ) then
            angle = 0 - math.atan2( ey - y, ex - x )
        end
    end

    vel_x = math.cos( angle + angle_random ) * length
    vel_y = 0- math.sin( angle + angle_random ) * length
    shoot_projectile( entity_id, "data/entities/misc/perks/revenge_tentacle_tentacle.xml", x, y, vel_x, vel_y )
end