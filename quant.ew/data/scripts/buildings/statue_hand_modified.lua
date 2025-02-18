dofile_once("data/scripts/lib/utilities.lua")

function mark_as_complete()
    local entity_id = GetUpdatedEntityID()
    local pos_x, pos_y = EntityGetTransform(entity_id)

    local function disappear(count)
        EntityKill(entity_id)
        EntityLoad("data/entities/buildings/statue_hand_fx.xml", pos_x, pos_y)
        for i = 1, count do
            EntityLoad("data/entities/animals/monk.xml", pos_x + i % 2 * 4, pos_y - (i - 1) * 10)
        end
    end

    if EntityHasTag(entity_id, "statue_hand_3") and GameHasFlagRun("statue_hands_destroyed_2") then
        GameAddFlagRun("statue_hands_destroyed_3")
        --print("3rd statue destroyed at " .. pos_x .. ", " .. pos_y)
        --CreateItemActionEntity( "SUMMON_PORTAL_BROKEN", pos_x, pos_y - 5)
        EntityLoad("data/entities/items/pickup/summon_portal_broken.xml", pos_x, pos_y - 5)
        disappear(3)
    elseif EntityHasTag(entity_id, "statue_hand_2") and GameHasFlagRun("statue_hands_destroyed_1") then
        GameAddFlagRun("statue_hands_destroyed_2")
        --print("2nd statue destroyed at " .. pos_x .. ", " .. pos_y)
        disappear(2)
    elseif EntityHasTag(entity_id, "statue_hand_1") then
        GameAddFlagRun("statue_hands_destroyed_1")
        --print("1st statue destroyed at " .. pos_x .. ", " .. pos_y)
        disappear(1)
    end
end

function physics_body_modified(is_destroyed)
    --print(tostring(is_destroyed))
    --if is_destroyed then mark_as_complete() end
end

function kick()
    local entity_id = GetUpdatedEntityID()
    local util = dofile_once("mods/quant.ew/files/resource/util_min.lua")
    if util.do_i_own(entity_id) then
        mark_as_complete()
    end
    --print("statue kicked")
end
