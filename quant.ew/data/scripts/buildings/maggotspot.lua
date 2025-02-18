dofile_once("data/scripts/lib/utilities.lua")
local util = dofile_once("mods/quant.ew/files/resource/util_min.lua")

local entity_id = GetUpdatedEntityID()
if not util.do_i_own(entity_id) then
    EntityKill(entity_id)
    return
end
local pos_x, pos_y = EntityGetTransform(entity_id)

local t = EntityGetInRadiusWithTag(pos_x, pos_y, 220, "ew_peer")

if #t > 0 then
    EntityLoad("data/entities/animals/maggot_tiny/maggot_tiny.xml", pos_x, pos_y)
    EntityLoad("data/entities/particles/image_emitters/magical_symbol_fast.xml", pos_x, pos_y)
    EntityKill(entity_id)
end
