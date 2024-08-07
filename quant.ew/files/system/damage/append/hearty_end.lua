local entity_id = GetUpdatedEntityID()
local player_id = EntityGetRootEntity( entity_id )

if player_id ~= 0 and EntityHasTag( player_id, "player_unit" ) then
    CrossCall("ew_ds_effect_hearty", false)
else
    dofile("scripts/status_effects/hearty_end.lua")
end
