local entity_id = GetUpdatedEntityID()
local sx, sy = EntityGetTransform( entity_id )
CrossCall("ew_ending_sequence", sx, sy)
