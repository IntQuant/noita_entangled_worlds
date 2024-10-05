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

local old_shield_on = shield_on
local old_shield_off = shield_off

local shield_enabled = false

function shield_on()
    local newgame_n = tonumber( SessionNumbersGetValue("NEW_GAME_PLUS_COUNT") )
    local orbcount = GameGetOrbCountThisRun() + newgame_n
    if not shield_enabled then
        CrossCall("ew_kolmi_shield", true, orbcount)
        shield_enabled = true
    end
    return old_shield_on()
end

function shield_off()
    local newgame_n = tonumber( SessionNumbersGetValue("NEW_GAME_PLUS_COUNT") )
    local orbcount = GameGetOrbCountThisRun() + newgame_n
    if shield_enabled then
        CrossCall("ew_kolmi_shield", false, orbcount)
        shield_enabled = false
    end
    return old_shield_off()
end