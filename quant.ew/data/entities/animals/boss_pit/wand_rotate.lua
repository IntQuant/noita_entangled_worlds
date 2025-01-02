dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local x, y = EntityGetTransform(GetUpdatedEntityID())

local players = EntityGetWithTag("ew_peer")

local closest
local dir = 0
for _, player in ipairs(players) do
    if not EntityHasTag(player, "ew_notplayer") then
        local px, py = EntityGetTransform(player)
        local r = px * px + py * py
        if closest == nil or r < closest then
            closest = r
            dir = math.atan2(py - y, px - x)
        end
    end
end

EntitySetTransform(entity_id, x, y, dir)
