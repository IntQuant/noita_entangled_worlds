dofile_once("data/scripts/lib/utilities.lua")
function damage_received(damage, desc, entity_who_caused, is_fatal)
    local entity_id = GetUpdatedEntityID()
    local var = EntityGetFirstComponentIncludingDisabled(entity_id, "VariableStorageComponent", "ew_damage_tracker")
    if var ~= nil then
        local dtype = GetDamageDetails().damage_types
        ComponentSetValue2(var, "value_int", dtype)
    end
end
