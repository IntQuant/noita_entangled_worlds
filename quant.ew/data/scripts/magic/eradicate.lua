dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local x, y = EntityGetTransform(entity_id)

local targets = EntityGetWithTag("mortal")
for i, v in ipairs(targets) do
    if EntityHasTag(v, "ew_peer") == false and not EntityHasTag(v, "ew_notplayer") then
        local test = EntityGetFirstComponent(v, "DamageModelComponent")

        if test ~= nil then
            if not EntityHasTag(v, "ew_client") and not EntityHasTag(v, "polymorphed_player") then
                EntityKill(v)
            else
                EntityInflictDamage(v, 1000000, "DAMAGE_CURSE", "", "NONE", 0, 0, GameGetWorldStateEntity())
            end
        end
    end
end
