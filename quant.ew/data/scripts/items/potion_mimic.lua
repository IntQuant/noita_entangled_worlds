dofile_once("data/scripts/lib/utilities.lua")

function spawn_leggy(entity_item)
    if not GameHasFlagRun("ew_flag_this_is_host") then
        return
    end
    local x, y = EntityGetTransform(entity_item)
    EntityLoad("data/entities/particles/polymorph_explosion.xml", x, y)
    GamePlaySound("data/audio/Desktop/misc.bank", "game_effect/polymorph/create", x, y)
    EntityLoad("data/entities/animals/mimic_potion.xml", x, y)
    EntityKill(entity_item)
end

function item_pickup(entity_item, entity_who_picked, name)
    spawn_leggy(entity_item)
end

function physics_body_modified(is_destroyed)
    local entity_item = GetUpdatedEntityID()
    spawn_leggy(entity_item)
end

function collision_trigger(colliding_entity_id)
    local entity_item = GetUpdatedEntityID()
    spawn_leggy(entity_item)
end
