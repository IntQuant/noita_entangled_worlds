local old_entity_load = EntityLoad

function EntityLoad(filename, x, y)
    if filename == "data/entities/buildings/teleport_ending_victory_delay.xml" then
        CrossCall("ew_kolmi_spawn_portal", x, y)
    end
    return old_entity_load(filename, x, y)
end

local old_main_anim = set_main_animation

function set_main_animation(current_name, next_name)
    old_main_anim(current_name, next_name) -- Doesn't return anything
    CrossCall("ew_kolmi_anim", current_name, next_name, is_aggro)
end