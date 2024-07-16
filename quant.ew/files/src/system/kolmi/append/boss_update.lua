local old_entity_load = EntityLoad

function EntityLoad(filename, x, y)
    if filename == "data/entities/buildings/teleport_ending_victory_delay.xml" then
        CrossCall("ew_kolmi_spawn_portal", x, y)
    end
    return old_entity_load(filename, x, y)
end