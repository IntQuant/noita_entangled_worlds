dofile_once("data/scripts/lib/utilities.lua")
function collision_trigger()
    local entity_id = GetUpdatedEntityID()
    CrossCall("ew_spawn_dragon_boss", entity_id)
end
