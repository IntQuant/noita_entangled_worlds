dofile_once("data/scripts/lib/utilities.lua")
dofile_once("data/scripts/biomes/temple_shared.lua")

function collision_trigger()
    local entity_id = GetUpdatedEntityID()
    local pos_x, pos_y = EntityGetTransform(entity_id)

    -- this reset the biome stats - see workshop_trigger_check_stats.lua
    StatsBiomeReset()

    EntityLoad("data/entities/particles/image_emitters/magical_symbol.xml", pos_x - 144, pos_y + 82 - 12)
    --EntityLoad("data/entities/misc/workshop_collapse.xml", pos_x-144, pos_y+82)
    --EntityLoad("data/entities/misc/workshop_areadamage.xml", pos_x-143, pos_y+47)
    --EntityLoad("data/entities/misc/workshop_areadamage.xml", pos_x-543, pos_y+47)

    -- Destroy workshop entities

    local workshop_1 = EntityGetClosestWithTag(pos_x, pos_y, "workshop")
    EntityKill(workshop_1)
    --print("Destroyed workshop entity 2")

    local workshop_2 = EntityGetClosestWithTag(pos_x, pos_y, "workshop")
    EntityKill(workshop_2)
    --print("Destroyed workshop entity 2")

    local workshop_2b = EntityGetClosestWithTag(pos_x, pos_y, "workshop")
    EntityKill(workshop_2b)
    --print("Destroyed workshop entity 2b")

    local workshop_3 = EntityGetClosestWithTag(pos_x, pos_y, "workshop_show_hint")
    EntityKill(workshop_3)
    --print("Destroyed workshop hint entity")

    -- kill temple_areacheckers that are on (about) the same horizontal level as we are
    local temple_areacheckers = EntityGetInRadiusWithTag(pos_x, pos_y, 2048, "temple_areachecker")
    for k, areachecker in pairs(temple_areacheckers) do
        local area_x, area_y = EntityGetTransform(areachecker)
        if math.abs(pos_y - area_y) < 512 then
            EntityKill(areachecker)
        end
    end

    -- SetGlobalValue( "temple_collapsed_" )
    -- tags="temple_areachecker"

    local collapse_name = "TEMPLE_COLLAPSED_" .. temple_pos_to_id(pos_x, pos_y)
    GlobalsSetValue(collapse_name, "1")

    temple_set_active_flag(pos_x, pos_y, "0")

    GameTriggerMusicFadeOutAndDequeueAll(2.0)
    GamePlaySound("data/audio/Desktop/misc.bank", "misc/temple_collapse", pos_x - 112, pos_y + 40)
end
