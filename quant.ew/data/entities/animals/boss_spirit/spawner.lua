dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local x, y = EntityGetTransform(GetUpdatedEntityID())

local anger = tonumber(GlobalsGetValue("HELPLESS_KILLS", "1")) or 1

local p = EntityGetInRadiusWithTag(x, y, 200, "ew_peer")

if (#p > 0) and (anger >= 30) and (GlobalsGetValue("ISLANDSPIRIT_SPAWNED", "0") == "0") then
    GlobalsSetValue("ISLANDSPIRIT_SPAWNED", "1")

    if CrossCall("ew_do_i_own", entity_id) then
        EntityLoad("data/entities/animals/boss_spirit/spawn_portal.xml", x, y)
    else
        EntityLoad("mods/quant.ew/files/system/tapion/spawn_portal.xml", x, y)
    end
    EntityKill(entity_id)
end
