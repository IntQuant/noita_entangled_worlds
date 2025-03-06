local function early_init()
    if #ModLuaFileGetAppends("mods/quant.ew/files/core/early_init.lua") == 0 then
        -- Use appends to store data
        ModLuaFileAppend("mods/quant.ew/files/core/early_init.lua", "data/scripts/empty.lua")

        -- Early init stuff, called before main "mod" is loaded. Meaning we can append to data/scripts/init.lua
        dofile("mods/quant.ew/files/core/early_init.lua")
    end
end

dofile("data/scripts/lib/mod_settings.lua") -- see this file for documentation on some of the features.

-- This file can't access other files from this or other mods in all circumstances.
-- Settings will be automatically saved.
-- Settings don't have access unsafe lua APIs.

-- Use ModSettingGet() in the game to query settings.
-- For some settings (for example those that affect world generation) you might want to retain the current value until a certain point, even
-- if the player has changed the setting while playing.
-- To make it easy to define settings like that, each setting has a "scope" (e.g. MOD_SETTING_SCOPE_NEW_GAME) that will define when the changes
-- will actually become visible via ModSettingGet(). In the case of MOD_SETTING_SCOPE_NEW_GAME the value at the start of the run will be visible
-- until the player starts a new game.
-- ModSettingSetNextValue() will set the buffered value, that will later become visible via ModSettingGet(), unless the setting scope is MOD_SETTING_SCOPE_RUNTIME.

function mod_setting_change_callback(mod_id, gui, in_main_menu, setting, old_value, new_value)
    print(tostring(new_value))
end

local mod_id = "quant.ew" -- This should match the name of your mod's folder.
local prfx = mod_id .. "."
mod_settings_version = 1 -- This is a magic global that can be used to migrate settings to new mod versions. call mod_settings_get_version() before mod_settings_update() to get the old value.
mod_settings = {}

--KEY SWITCHER IS FROM NOITA FAIR MOD <3 thx
--- gather keycodes from game file
local function gather_key_codes()
    local arr = {}
    arr["0"] = GameTextGetTranslatedOrNot("$menuoptions_configurecontrols_action_unbound")
    local keycodes_all = ModTextFileGetContent("data/scripts/debug/keycodes.lua")
    for line in keycodes_all:gmatch("Key_.-\n") do
        local _, key, code = line:match("(Key_)(.+) = (%d+)")
        arr[code] = key:upper()
    end
    return arr
end
local keycodes = gather_key_codes()

local function pending_input()
    for code, _ in pairs(keycodes) do
        if InputIsKeyJustDown(code) then
            return code
        end
    end
end

local function ui_get_input(_, gui, _, im_id, setting)
    local setting_id = prfx .. setting.id
    local current = tostring(ModSettingGetNextValue(setting_id)) or "0"
    local current_key = "[" .. keycodes[current] .. "]"

    if setting.is_waiting_for_input then
        current_key = GameTextGetTranslatedOrNot("$menuoptions_configurecontrols_pressakey")
        local new_key = pending_input()
        if new_key then
            ModSettingSetNextValue(setting_id, new_key, false)
            setting.is_waiting_for_input = false
        end
    end

    GuiLayoutBeginHorizontal(gui, 0, 0, true, 0, 0)
    GuiText(gui, mod_setting_group_x_offset, 0, setting.ui_name)

    GuiText(gui, 8, 0, "")
    local _, _, _, x, y = GuiGetPreviousWidgetInfo(gui)
    local w, h = GuiGetTextDimensions(gui, current_key)
    GuiOptionsAddForNextWidget(gui, GUI_OPTION.ForceFocusable)
    GuiImageNinePiece(gui, im_id, x, y, w, h, 0)
    local _, _, hovered = GuiGetPreviousWidgetInfo(gui)
    if hovered then
        GuiTooltip(gui, setting.ui_description, GameTextGetTranslatedOrNot("$menuoptions_reset_keyboard"))
        GuiColorSetForNextWidget(gui, 1, 1, 0.7, 1)
        if InputIsMouseButtonJustDown(1) then
            setting.is_waiting_for_input = true
        end
        if InputIsMouseButtonJustDown(2) or InputIsMouseButtonJustDown(3) then
            GamePlaySound("ui", "ui/button_click", 0, 0)
            ModSettingSetNextValue(setting_id, setting.value_default, false)
            setting.is_waiting_for_input = false
        end
    end
    if keycodes[current] == "BACKSPACE" then
        current_key = "[MIDDLE MOUSE BUTTON]"
    end
    GuiText(gui, 0, 0, current_key)

    GuiLayoutEnd(gui)
end

local function build_settings()
    local settings = {
        {
            category_id = "keybinds",
            ui_name = "keybinds",
            ui_description = "keybinds",
            settings = {
                {
                    id = "rebind_ping",
                    ui_name = "Ping button",
                    ui_description = "ping",
                    value_default = "42",
                    ui_fn = ui_get_input,
                    is_waiting_for_input = false,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "rebind_lspectate",
                    ui_name = "spectate left Button",
                    ui_description = "left",
                    value_default = "54",
                    ui_fn = ui_get_input,
                    is_waiting_for_input = false,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "rebind_rspectate",
                    ui_name = "spectate right Button",
                    ui_description = "right",
                    value_default = "55",
                    ui_fn = ui_get_input,
                    is_waiting_for_input = false,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "rebind_sspectate",
                    ui_name = "spectate self Button",
                    ui_description = "self",
                    value_default = "56",
                    ui_fn = ui_get_input,
                    is_waiting_for_input = false,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "rebind_mspectate",
                    ui_name = "spectate near Button",
                    ui_description = "near",
                    value_default = "52",
                    ui_fn = ui_get_input,
                    is_waiting_for_input = false,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "rebind_ptt",
                    ui_name = "push to talk",
                    ui_description = "push to talk, options in proxy VoIP",
                    value_default = "23",
                    ui_fn = ui_get_input,
                    is_waiting_for_input = false,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "ptt_toggle",
                    ui_name = "toggle mute",
                    ui_description = "toggle mute instead of hold to push to talk",
                    value_default = false,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "text",
                    ui_name = "text",
                    ui_description = "hi",
                    value_default = "40",
                    ui_fn = ui_get_input,
                    is_waiting_for_input = false,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "stoptext",
                    ui_name = "stop text",
                    ui_description = "bye",
                    value_default = "76",
                    ui_fn = ui_get_input,
                    is_waiting_for_input = false,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "no_gamepad",
                    ui_name = "dont add keybinds for gamepad",
                    ui_description = ":(",
                    value_default = false,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
            },
        },
        {
            category_id = "ui",
            ui_name = "ui",
            ui_description = "ui",
            settings = {
                {
                    id = "notext",
                    ui_name = "no text",
                    ui_description = ":c",
                    value_default = false,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "nochathint",
                    ui_name = "no chat hint",
                    ui_description = ":c",
                    value_default = true,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "textcolor",
                    ui_name = "chat nickname brightening",
                    ui_description = "0 - player primary color, 255 - white, default - 63",
                    value_default = 0,
                    value_min = 0,
                    value_max = 255,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "textaltcolor",
                    ui_name = "chat message brightening",
                    ui_description = "0 - player secondary color, 255 - white, default - 191",
                    value_default = 255,
                    value_min = 0,
                    value_max = 255,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "ping_life",
                    ui_name = "ping lifetime",
                    ui_description = "in seconds",
                    value_default = "6.0",
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "ping_size",
                    ui_name = "ping extra size",
                    ui_description = "in pixels",
                    value_default = "0.0",
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "disable_cursors",
                    ui_name = "disable others cursors",
                    ui_description = "bool",
                    value_default = false,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "disable_arrows",
                    ui_name = "disable others arrows",
                    ui_description = "bool",
                    value_default = false,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "disable_nametags",
                    ui_name = "disable others name tags",
                    ui_description = "bool",
                    value_default = false,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
            },
        },
        {
            category_id = "misc",
            ui_name = "misc",
            ui_description = "misc",
            settings = {
                {
                    id = "flex",
                    ui_name = "flexible updates",
                    ui_description = "some esotaric chunk thing, may help performance, buggy",
                    value_default = false,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "text_range",
                    ui_name = "text chat range",
                    ui_description = "range in pixels for text chat, 0 for disabled",
                    value_default = "0",
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "tetherrange",
                    ui_name = "tether range",
                    ui_description = "radius in pixels, 0 for disabled",
                    value_default = "0",
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "entity_sync",
                    ui_name = "entity sync interval",
                    ui_description = "every N frames entitys under your authority are synced",
                    value_default = "2",
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "world_sync",
                    ui_name = "world sync interval",
                    ui_description = "rate at which world is synced~",
                    value_default = "4",
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                --[[                {
                    id = "rocks",
                    ui_name = "cap of special item entities to be synced",
                    ui_description = "max amount of proj to be synced, -1 for infinite",
                    value_default = "16",
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },TODO]]
                {
                    id = "explosions",
                    ui_name = "amount of radii of explosions can be handled in 1 frame",
                    ui_description = "decrease if weird network lag, increase if weird world sync, cpu bounded",
                    value_default = "128",
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "cell_eater",
                    ui_name = "amount of length of cell eater logic can be handled in 1 frame",
                    ui_description = "decrease if weird network lag, increase if weird world sync, cpu bounded",
                    value_default = "64",
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "disable_shield",
                    ui_name = "disable shield on death",
                    ui_description = "bool",
                    value_default = false,
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
                {
                    id = "team",
                    ui_name = "friendly fire team",
                    ui_description = "team for friendly fire, 0 no team, -1 friendly",
                    value_default = "0",
                    scope = MOD_SETTING_SCOPE_RUNTIME,
                },
            },
        },
    }
    return settings
end
mod_settings = build_settings()

-- This function is called to ensure the correct setting values are visible to the game via ModSettingGet(). your mod's settings don't work if you don't have a function like this defined in settings.lua.
-- This function is called:
--        - when entering the mod settings menu (init_scope will be MOD_SETTINGS_SCOPE_ONLY_SET_DEFAULT)
--         - before mod initialization when starting a new game (init_scope will be MOD_SETTING_SCOPE_NEW_GAME)
--        - when entering the game after a restart (init_scope will be MOD_SETTING_SCOPE_RESTART)
--        - at the end of an update when mod settings have been changed via ModSettingsSetNextValue() and the game is unpaused (init_scope will be MOD_SETTINGS_SCOPE_RUNTIME)
function ModSettingsUpdate(init_scope)
    --local old_version = mod_settings_get_version( mod_id ) -- This can be used to migrate some settings between mod versions.
    mod_settings = build_settings()
    mod_settings_update(mod_id, mod_settings, init_scope)
    if ModIsEnabled(mod_id) and (init_scope == 0 or init_scope == 1) then
        print("Running early init fn")
        early_init()
    end
end

-- This function should return the number of visible setting UI elements.
-- Your mod's settings wont be visible in the mod settings menu if this function isn't defined correctly.
-- If your mod changes the displayed settings dynamically, you might need to implement custom logic.
-- The value will be used to determine whether or not to display various UI elements that link to mod settings.
-- At the moment it is fine to simply return 0 or 1 in a custom implementation, but we don't guarantee that will be the case in the future.
-- This function is called every frame when in the settings menu.
function ModSettingsGuiCount()
    return mod_settings_gui_count(mod_id, mod_settings)
end

-- This function is called to display the settings UI for this mod. Your mod's settings wont be visible in the mod settings menu if this function isn't defined correctly.
function ModSettingsGui(gui, in_main_menu)
    mod_settings_gui(mod_id, mod_settings, gui, in_main_menu)
end
