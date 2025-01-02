function kick(entity_who_kicked)
    CrossCall("ew_kicked_orb", GetUpdatedEntityID(), entity_who_kicked, true)
end
