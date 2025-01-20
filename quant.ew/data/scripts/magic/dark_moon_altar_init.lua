if HasFlagPersistent("darkmoon_is_darksun") then
    local entity_id = GetUpdatedEntityID()
    local x, y = EntityGetTransform(entity_id)
    CrossCall("ew_moon_spawn", x, y)
end
