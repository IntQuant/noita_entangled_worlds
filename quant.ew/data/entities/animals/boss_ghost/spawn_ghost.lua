dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local root_id = EntityGetRootEntity(entity_id)
local x, y = EntityGetTransform(root_id)

local players = EntityGetInRadiusWithTag(x, y, 300, "ew_peer")

if #players > 0 then
    EntityLoad("data/entities/animals/boss_ghost/boss_ghost.xml", x, y)
    EntityKill(root_id)
end
