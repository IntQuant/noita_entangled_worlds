dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local x, y = EntityGetTransform(entity_id)

SetRandomSeed(x, y)
local rats = EntityGetWithTag("perk_fungus_tiny")

if #rats < 30 then
    if
        (Random(1, 20) == 5)
        and EntityGetClosestWithTag(x, y, "ew_peer") == EntityGetClosestWithTag(x, y, "player_unit")
    then
        local eid = EntityLoad("data/entities/misc/perks/fungus.xml", x, y)
        EntityRemoveTag(eid, "enemy")
    end
end

EntityKill(entity_id)
