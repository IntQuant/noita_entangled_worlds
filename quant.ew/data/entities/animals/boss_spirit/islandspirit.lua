dofile_once("data/scripts/lib/utilities.lua")

function damage_received( damage, msg, source )
    local entity_id = GetUpdatedEntityID()

    local anger = tonumber( GlobalsGetValue( "HELPLESS_KILLS", "1" ) ) or 1
    local dmg = damage * anger * 0.1

    local player = EntityGetWithTag( "player_unit" )

    for i,v in ipairs( player ) do
        if not EntityHasTag(v, "ew_notplayer") then
            EntityInflictDamage( v, dmg, "DAMAGE_CURSE", "$animal_islandspirit", "DISINTEGRATED", 0, 0, entity_id )
        end
    end
end

function death( damage_type_bit_field, damage_message, entity_thats_responsible, drop_items )
    local entity_id = GetUpdatedEntityID()
    local x, y = EntityGetTransform( entity_id )

    CreateItemActionEntity( "MASS_POLYMORPH", x, y )

    AddFlagPersistent( "card_unlocked_polymorph" )
    AddFlagPersistent( "miniboss_islandspirit" )

    GlobalsSetValue( "BOSS_SPIRIT_DEAD", "1" )

    local anger = tonumber( GlobalsGetValue( "HELPLESS_KILLS", "1" ) ) or 1
    if ( anger >= 300 ) then
        AddFlagPersistent( "miniboss_threelk" )
    end
end


-- every update
local anger = tonumber( GlobalsGetValue( "HELPLESS_KILLS", "1" ) ) or 1

-- only remove ambrosia if player has murdered more than 60 helpless animals
if( anger >= 60 ) then
    local players = get_players()
    if #players > 0 then
        local player_id = players[1]
        EntityRemoveStainStatusEffect( player_id, "PROTECTION_ALL", 5 )
    end
end

VerletApplyCircularForce( x, y, 80, 0.14 )