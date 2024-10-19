dofile_once("data/scripts/lib/utilities.lua")
dofile_once("mods/quant.ew/files/resource/shoot_projectile_fix.lua")

function damage_received( damage, desc, entity_who_caused, is_fatal )
    local entity_id    = GetUpdatedEntityID()
    local pos_x, pos_y = EntityGetTransform( entity_id )

    if( damage < 0 ) then return end
    if entity_who_caused == 0 or entity_who_caused == 1 or ( entity_who_caused == entity_id ) or ( ( EntityGetParent( entity_id ) ~= NULL_ENTITY ) and ( entity_who_caused == EntityGetParent( entity_id ) ) ) then return end

    -- check that we're only shooting every 10 frames
    if script_wait_frames( entity_id, 5 ) then  return  end
    local eid = shoot_projectile( entity_id, "data/entities/misc/perks/revenge_explosion.xml", pos_x, pos_y, 0, 0 )

    local herd_id = -1
    edit_component( entity_id, "GenomeDataComponent", function(comp,vars)
        herd_id = ComponentGetValue2( comp, "herd_id" )
    end)

    local new_ent = entity_id

    if EntityHasTag(entity_id, "ew_client") and not EntityHasTag(entity_id, "ew_notplayer") then
        new_ent = EntityGetWithTag("player_unit")[1]
    end

    edit_component( eid, "ProjectileComponent", function(comp,vars)
        ComponentSetValue2(comp, "mWhoShot", new_ent)
        ComponentSetValue2(comp, "mShooterHerdId", herd_id)

        ComponentObjectSetValue(comp, "config_explosion", "dont_damage_this", new_ent)
    end)
end