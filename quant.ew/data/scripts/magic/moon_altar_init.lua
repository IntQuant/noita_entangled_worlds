if HasFlagPersistent("moon_is_sun") then
    local entity_id = GetUpdatedEntityID()
    local x, y = EntityGetTransform(entity_id)
    CrossCall("ew_moon_spawn", x, y)
end
