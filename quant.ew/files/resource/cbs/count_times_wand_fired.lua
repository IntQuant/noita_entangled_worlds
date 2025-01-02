function wand_fired(gun_entity_id)
    local current = tonumber(GlobalsGetValue("ew_wand_fired", "0"))
    GlobalsSetValue("ew_wand_fired", tostring(current + 1))
end
