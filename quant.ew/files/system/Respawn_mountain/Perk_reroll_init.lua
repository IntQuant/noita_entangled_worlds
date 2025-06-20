local entity_id = GetUpdatedEntityID()
local x,y = EntityGetTransform(entity_id)

CrossCall("ew_spawn_ankh_anchor", x-150, y+4)