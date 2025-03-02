dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local x, y = EntityGetTransform(entity_id)

local targets = EntityGetWithTag("mortal")
for i, v in ipairs(targets) do
    if EntityHasTag(v, "ew_peer") == false and not EntityHasTag(v, "ew_notplayer") then
        local test = EntityGetFirstComponent(v, "DamageModelComponent")

        if test ~= nil then
            EntityKill(v)
        end
    end
end
