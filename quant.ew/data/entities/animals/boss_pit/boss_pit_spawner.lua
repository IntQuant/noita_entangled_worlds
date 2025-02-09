dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local x, y = EntityGetTransform(entity_id)

SetRandomSeed(x, y)

if
    (Random(1, 10) == 5)
    and EntityGetClosestWithTag(x, y, "ew_peer") == EntityGetClosestWithTag(x, y, "player_unit")
then
    EntityLoad("data/entities/animals/boss_pit/boss_pit.xml", x, y)
end

EntityKill(entity_id)
