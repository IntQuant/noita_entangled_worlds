-- #########################################
-- #######       EZWand v2.2.2       #######
-- #########################################

dofile_once("data/scripts/gun/procedural/gun_action_utils.lua")
dofile_once("data/scripts/gun/gun_enums.lua")
dofile_once("data/scripts/lib/utilities.lua")
dofile_once("data/scripts/gun/procedural/wands.lua")

-- ##########################
-- ####       UTILS      ####
-- ##########################

if not spell_lookup then
  spell_lookup = {}
  dofile_once("data/scripts/gun/gun_actions.lua")
  for i, action in ipairs(actions) do
    spell_lookup[action.id] = {
      icon = action.sprite,
      type = action.type
    }
  end
end

-- Removes spells from a table whose ID is not found in the gun_actions table
local function filter_spells(spells)
  dofile_once("data/scripts/gun/gun_actions.lua")
  if not spell_exist_lookup then
    spell_exist_lookup = {}
    for i, v in ipairs(actions) do
      spell_exist_lookup[v.id] = true
    end
  end
  local out = {}
  for i, spell in ipairs(spells) do
    if spell == "" or spell_exist_lookup[spell] then
      table.insert(out, spell)
    end
  end
  return out
end

local function string_split(inputstr, sep)
  sep = sep or "%s"
  local t= {}
  local pos = 0
  local function next(s)
    pos = pos + 1
    local out = s:sub(pos, pos)
    if out ~= "" then
      return out
    end
  end
  local cur_str = ""
  local next_char = next(inputstr)
  while next_char do
    if next_char == sep then
      table.insert(t, cur_str)
      next_char = next(inputstr)
      cur_str = ""
    else
      cur_str = cur_str .. next_char
      next_char = next(inputstr)
    end
  end
  table.insert(t, cur_str)
  return t
end

local function test_conditionals(conditions)
  for i, conditon in ipairs(conditions) do
    if not conditon[1] then
      return false, conditon[2]
    end
  end
  return true
end

wand_props = {
  shuffle = {
    validate = function(val)
      return test_conditionals{
        { type(val) == "boolean", "shuffle must be true or false" }
      }
    end,
    default = false,
  },
  spellsPerCast = {
    validate = function(val)
      return test_conditionals{
        { type(val) == "number", "spellsPerCast must be a number" },
        { val > 0, "spellsPerCast must be a number > 0" },
      }
    end,
    default = 1,
  },
  castDelay = {
    validate = function(val)
      return test_conditionals{
        { type(val) == "number", "castDelay must be a number" },
      }
    end,
    default = 20,
  },
  currentCastDelay = {
    validate = function(val)
      return test_conditionals{
        { type(val) == "number", "currentCastDelay must be a number" },
      }
    end,
  },
  rechargeTime = {
    validate = function(val)
      return test_conditionals{
        { type(val) == "number", "rechargeTime must be a number" },
      }
    end,
    default = 40,
  },
  currentRechargeTime = {
    validate = function(val)
      return test_conditionals{
        { type(val) == "number", "currentRechargeTime must be a number" },
      }
    end,
  },
  manaMax = {
    validate = function(val)
      return test_conditionals{
        { type(val) == "number", "manaMax must be a number" },
        { val >= 0, "manaMax must be a number > 0" },
      }
    end,
    default = 500,
  },
  mana = {
    validate = function(val)
      return test_conditionals{
        { type(val) == "number", "mana must be a number" },
        { val >= 0, "mana must be a number > 0" },
      }
    end,
    default = 500,
  },
  manaChargeSpeed = {
    validate = function(val)
      return test_conditionals{
        { type(val) == "number", "manaChargeSpeed must be a number" },
      }
    end,
    default = 200,
  },
  capacity = {
    validate = function(val)
      return test_conditionals{
        { type(val) == "number", "capacity must be a number" },
        { val >= 0, "capacity must be a number >= 0" },
      }
    end,
    default = 10,
  },
  spread = {
    validate = function(val)
      return test_conditionals{
        { type(val) == "number", "spread must be a number" },
      }
    end,
    default = 10,
  },
  speedMultiplier = {
    validate = function(val)
      return test_conditionals{
        { type(val) == "number", "speedMultiplier must be a number" },
      }
    end,
    default = 1,
  },
}
-- Throws an error if the value doesn't have the correct format or the property doesn't exist
local function validate_property(name, value)
  if wand_props[name] == nil then
    error(name .. " is not a valid wand property.", 4)
  end
  local success, err = wand_props[name].validate(value)
  if not success then
    error(err, 4)
  end
end

--[[
  values is a table that contains info on what values to set
  example:
  values = {
    manaMax = 50,
    rechargeSpeed = 20
  }
  etc
  calls error() if values contains invalid properties
  fills in missing properties with default values
]]

function validate_wand_properties(values)
  if type(values) ~= "table" then
    error("Arg 'values': table expected.", 2)
  end
  -- Check if all passed in values are valid wand properties and have the required type
  for k, v in pairs(values) do
    validate_property(k, v)
  end
  -- Fill in missing properties with default values
  for k,v in pairs(wand_props) do
    values[k] = values[k] or v.default
  end
  return values
end

 function table.contains(table, element)
  for _, value in pairs(table) do
    if value == element then
      return true
    end
  end
  return false
end

-- Returns true if entity is a wand
local function entity_is_wand(entity_id)
    local ability_component = EntityGetFirstComponentIncludingDisabled(entity_id, "AbilityComponent")
    return ComponentGetValue2(ability_component, "use_gun_script") == true
end

local function starts_with(str, start)
  return str:match("^" .. start) ~= nil
end

local function ends_with(str, ending)
  return ending == "" or str:sub(-#ending) == ending
end

-- Parses a serialized wand string into a table with it's properties
local function deserialize_v1(str)
  local values = string_split(str, ";")
  if #values ~= 18 then
    error("Wrong wand import string format")
  end

  local out = {
    props = {
      shuffle = values[2] == "1",
      spellsPerCast = tonumber(values[3]),
      castDelay = tonumber(values[4]),
      rechargeTime = tonumber(values[5]),
      manaMax = tonumber(values[6]),
      mana = tonumber(values[7]),
      manaChargeSpeed = tonumber(values[8]),
      capacity = tonumber(values[9]),
      spread = tonumber(values[10]),
      speedMultiplier = tonumber(values[11])
    },
    spells = string_split(values[12] == "-" and "" or values[12], ","),
    always_cast_spells = string_split(values[13] == "-" and "" or values[13], ","),
    sprite_image_file = values[14],
    offset_x = tonumber(values[15]),
    offset_y = tonumber(values[16]),
    tip_x = tonumber(values[17]),
    tip_y = tonumber(values[18])
  }

  if #out.spells == 1 and out.spells[1] == "" then
    out.spells = {}
  end
  if #out.always_cast_spells == 1 and out.always_cast_spells[1] == "" then
    out.always_cast_spells = {}
  end

  return out
end

-- Parses a serialized wand string into a table with it's properties
local function deserialize_v2(str)
  local values = string_split(str, ";")
  if #values ~= 20 then
    error("Wrong wand import string format")
  end

  local out = {
    props = {
      shuffle = values[2] == "1",
      spellsPerCast = tonumber(values[3]),
      castDelay = tonumber(values[4]),
      rechargeTime = tonumber(values[5]),
      manaMax = tonumber(values[6]),
      mana = tonumber(values[7]),
      manaChargeSpeed = tonumber(values[8]),
      capacity = tonumber(values[9]),
      spread = tonumber(values[10]),
      speedMultiplier = tonumber(values[11])
    },
    spells = string_split(values[12] == "-" and "" or values[12], ","),
    always_cast_spells = string_split(values[13] == "-" and "" or values[13], ","),
    sprite_image_file = values[14],
    offset_x = tonumber(values[15]),
    offset_y = tonumber(values[16]),
    tip_x = tonumber(values[17]),
    tip_y = tonumber(values[18]),
    name = values[19],
    show_name_in_ui = (tonumber(values[20]) == 1) or false,
  }

  if #out.spells == 1 and out.spells[1] == "" then
    out.spells = {}
  end
  if #out.always_cast_spells == 1 and out.always_cast_spells[1] == "" then
    out.always_cast_spells = {}
  end

  return out
end

-- Parses a serialized wand string into a table with it's properties
local function deserialize(str)
  if not starts_with(str, "EZW") then
    error("Wrong wand import string format")
  end
  local this_version = 2
  local serialized_version = tonumber(str:match("EZWv(%d+)"))
  if serialized_version > this_version then
    return "Serialized wand was made with newer version of EZWand"
  end
  if serialized_version == 1 then
    return deserialize_v1(str)
  elseif serialized_version == 2 then
    return deserialize_v2(str)
  end
end


local spell_type_bgs = {
    [ACTION_TYPE_PROJECTILE] = "data/ui_gfx/inventory/item_bg_projectile.png",
    [ACTION_TYPE_STATIC_PROJECTILE] = "data/ui_gfx/inventory/item_bg_static_projectile.png",
    [ACTION_TYPE_MODIFIER] = "data/ui_gfx/inventory/item_bg_modifier.png",
    [ACTION_TYPE_DRAW_MANY] = "data/ui_gfx/inventory/item_bg_draw_many.png",
    [ACTION_TYPE_MATERIAL] = "data/ui_gfx/inventory/item_bg_material.png",
    [ACTION_TYPE_OTHER] = "data/ui_gfx/inventory/item_bg_other.png",
    [ACTION_TYPE_UTILITY] = "data/ui_gfx/inventory/item_bg_utility.png",
    [ACTION_TYPE_PASSIVE] = "data/ui_gfx/inventory/item_bg_passive.png",
}

local function get_spell_bg(action_id)
    return spell_type_bgs[spell_lookup[action_id] and spell_lookup[action_id].type] or spell_type_bgs[ACTION_TYPE_OTHER]
end

-- This function is a giant mess, but it works :)
-- wand needs to be of the same format as you get from EZWand.Deserialize():
-- {
--   props = {
--     shuffle = true,
--     spellsPerCast = 1,
--     castDelay = 30,
--     rechargeTime = 30,
--     manaMax = 200,
--     mana = 200,
--     manaChargeSpeed = 20,
--     capacity = 10,
--     spread = 0,
--     speedMultiplier = 1
--   },
--   spells = { "SPELL_ONE", "SPELL_TWO" },
--   always_cast_spells = { "SPELL_ONE", "SPELL_TWO" },
--   sprite_image_file = "data/whatever.png",
--   offset_x = 0,
--   offset_y = 0,
--   tip_x = 0,
--   tip_y = 0,
--   name = "Shootblaster",
--   show_name_in_ui = true
-- }
-- To get this easily you can use EZWand.Deserialize(EZWand(wand):Serialize())
-- Better cache it though, it's not super expensive but...
local function render_tooltip(origin_x, origin_y, wand, gui_, z)
  z = tonumber(z) or 0
  origin_x = tonumber(origin_x)
  if not origin_x then
    error("RenderTooltip: Argument x is required and must be a number", 2)
  end
  origin_y = tonumber(origin_y)
  if not origin_y then
    error("RenderTooltip: Argument y is required and must be a number", 2)
  end
  -- gui = gui or GuiCreate()
  gui = gui_ or gui or GuiCreate()
  if not gui_ then
    GuiStartFrame(gui)
  end
  GuiIdPushString(gui, "EZWand_tooltip")

  local margin = -3
  local wand_name = "WAND"
  local id = 1
  local function new_id()
    id = id + 1
    return id
  end
  local right = origin_x
  local bottom = origin_y
  local function update_bounds(rot)
    local _, _, _, x, y, w, h = GuiGetPreviousWidgetInfo(gui)
    if rot == -90 then
      local old_w = w
      w = h
      h = old_w
      y = y - h
    end
    right = math.max(right, x + w)
    bottom = math.max(bottom, y + h)
  end
  GuiLayoutBeginHorizontal(gui, origin_x, origin_y, true)
  GuiLayoutBeginVertical(gui, 0, 0)
  local text_lightness = 0.82
  local function gui_text_with_shadow(gui, x, y, text, lightness, z)
    z = z or 0
    lightness = lightness or text_lightness
    GuiColorSetForNextWidget(gui, lightness, lightness, lightness, 1)
    GuiZSetForNextWidget(gui, z - 1)
    GuiText(gui, x, y, text)
    GuiZSetForNextWidget(gui, z)
    GuiOptionsAddForNextWidget(gui, GUI_OPTION.Layout_NoLayouting)
    GuiColorSetForNextWidget(gui, 0, 0, 0, 0.83)
    local _, _, _, x, y = GuiGetPreviousWidgetInfo(gui)
    GuiText(gui, x, y + 1, text)
  end
  GuiColorSetForNextWidget(gui, text_lightness, text_lightness, text_lightness, 1)
  GuiZSetForNextWidget(gui, z - 1)
  GuiText(gui, 0, 0, wand_name)
  GuiZSetForNextWidget(gui, z - 1)
  GuiImage(gui, new_id(), 0, 4, "data/ui_gfx/inventory/icon_gun_shuffle.png", 1, 1, 1)
  GuiZSetForNextWidget(gui, z - 1)
  GuiImage(gui, new_id(), 0, 1, "data/ui_gfx/inventory/icon_gun_actions_per_round.png", 1, 1, 1)
  GuiZSetForNextWidget(gui, z - 1)
  GuiImage(gui, new_id(), 0, 1, "data/ui_gfx/inventory/icon_fire_rate_wait.png", 1, 1, 1)
  GuiZSetForNextWidget(gui, z - 1)
  GuiImage(gui, new_id(), 0, 1, "data/ui_gfx/inventory/icon_gun_reload_time.png", 1, 1, 1)
  GuiZSetForNextWidget(gui, z - 1)
  GuiImage(gui, new_id(), 0, 1, "data/ui_gfx/inventory/icon_mana_max.png", 1, 1, 1)
  GuiZSetForNextWidget(gui, z - 1)
  GuiImage(gui, new_id(), 0, 1, "data/ui_gfx/inventory/icon_mana_charge_speed.png", 1, 1, 1)
  GuiZSetForNextWidget(gui, z - 1)
  GuiImage(gui, new_id(), 0, 1, "data/ui_gfx/inventory/icon_gun_capacity.png", 1, 1, 1)
  GuiZSetForNextWidget(gui, z - 1)
  GuiImage(gui, new_id(), 0, 1, "data/ui_gfx/inventory/icon_spread_degrees.png", 1, 1, 1)
   -- Saves the position and width of the spread icon so we can draw the spells below it
  local _, _, _, last_icon_x, last_icon_y, last_icon_width, last_icon_height = GuiGetPreviousWidgetInfo(gui)
  GuiLayoutEnd(gui)
  local wand_name_width = GuiGetTextDimensions(gui, wand_name)
  GuiLayoutBeginVertical(gui, 12 - wand_name_width, -3, true)
  GuiText(gui, 0, 0, " ")
  gui_text_with_shadow(gui, 0, 5, GameTextGetTranslatedOrNot("$inventory_shuffle"), nil, z - 1)
  gui_text_with_shadow(gui, 0, margin, GameTextGetTranslatedOrNot("$inventory_actionspercast"), nil, z - 1)
  gui_text_with_shadow(gui, 0, margin, GameTextGetTranslatedOrNot("$inventory_castdelay"), nil, z - 1)
  gui_text_with_shadow(gui, 0, margin, GameTextGetTranslatedOrNot("$inventory_rechargetime"), nil, z - 1)
  gui_text_with_shadow(gui, 0, margin, GameTextGetTranslatedOrNot("$inventory_manamax"), nil, z - 1)
  gui_text_with_shadow(gui, 0, margin, GameTextGetTranslatedOrNot("$inventory_manachargespeed"), nil, z - 1)
  gui_text_with_shadow(gui, 0, margin, GameTextGetTranslatedOrNot("$inventory_capacity"), nil, z - 1)
  gui_text_with_shadow(gui, 0, margin, GameTextGetTranslatedOrNot("$inventory_spread"), nil, z - 1)
  GuiLayoutEnd(gui)
  GuiLayoutBeginVertical(gui, -6, -3, true)
  GuiText(gui, 0, 0, " ")
  local most_right_text_x = 0
  local function update_most_right_text_x()
    local _, _, _, x, y, w, h = GuiGetPreviousWidgetInfo(gui)
    most_right_text_x = math.max(most_right_text_x, x + w)
  end
  local function format_cast_delay_and_recharge_time(input)
    local pattern = "%.2f"
    if input % 1 == 0 then
      return input .. ".0 s"
    end
    return (pattern):format(input) .. " s"
  end
  gui_text_with_shadow(gui, 0, 5, GameTextGetTranslatedOrNot(wand.props.shuffle and "$menu_yes" or "$menu_no"), 1, z - 1)
  local _, _, _, _, no_text_y = GuiGetPreviousWidgetInfo(gui)
  update_most_right_text_x()
  gui_text_with_shadow(gui, 0, margin, ("%.0f"):format(wand.props.spellsPerCast), 1, z - 1)
  update_most_right_text_x()
  gui_text_with_shadow(gui, 0, margin, format_cast_delay_and_recharge_time(wand.props.castDelay / 60), 1, z - 1)
  update_most_right_text_x()
  gui_text_with_shadow(gui, 0, margin, format_cast_delay_and_recharge_time(wand.props.rechargeTime / 60), 1, z - 1)
  update_most_right_text_x()
  gui_text_with_shadow(gui, 0, margin, ("%.0f"):format(wand.props.manaMax), 1, z - 1)
  update_most_right_text_x()
  gui_text_with_shadow(gui, 0, margin, ("%.0f"):format(wand.props.manaChargeSpeed), 1, z - 1)
  update_most_right_text_x()
  gui_text_with_shadow(gui, 0, margin, ("%.0f"):format(wand.props.capacity), 1, z - 1)
  update_most_right_text_x()
  gui_text_with_shadow(gui, 0, margin, ("%.1f DEG"):format(wand.props.spread), 1, z - 1)
  update_most_right_text_x()
  update_bounds()
  local _, _, _, spread_text_x, spread_text_y, spread_text_width, spread_text_height = GuiGetPreviousWidgetInfo(gui)
  GuiLayoutEnd(gui)
  GuiLayoutEnd(gui)
  local always_cast_spell_icon_scale = 0.711
  local add_some = 0 -- I'm out of creativity for variable names...
  -- Always casts
  if type(wand.always_cast_spells) == "table" and #wand.always_cast_spells > 0 then
    add_some = 3
    local background_scale = 0.768
    GuiLayoutBeginHorizontal(gui, last_icon_x, last_icon_y + last_icon_height + 8, true)
    GuiZSetForNextWidget(gui, z - 1)
    GuiImage(gui, new_id(), 0, 1, "data/ui_gfx/inventory/icon_gun_permanent_actions.png", 1, 1, 1)
    _, _, _, last_icon_x, last_icon_y, last_icon_width, last_icon_height = GuiGetPreviousWidgetInfo(gui)
    gui_text_with_shadow(gui, 3, 0, GameTextGetTranslatedOrNot("$inventory_alwayscasts"), 1, z - 1)
    local _, _, _, ac_icon_x, ac_icon_y, ac_icon_width, ac_icon_height = GuiGetPreviousWidgetInfo(gui)
    local last_ac_x, last_ac_y, last_ac_width, last_ac_height
    for i, spell in ipairs(wand.always_cast_spells) do
      if i == 1 then
        update_bounds()
      end
      local item_bg_icon = get_spell_bg(spell)
      local w, h = GuiGetImageDimensions(gui, item_bg_icon, background_scale)
      local x, y
      if i == 1 then
        x, y = ac_icon_x + ac_icon_width + 3, math.floor(ac_icon_y - ac_icon_height / 2 + 2)
      else
        x, y = math.floor(last_ac_x + (last_ac_width - 2)) + 1, last_ac_y
      end
      GuiZSetForNextWidget(gui, z - 1)
      GuiOptionsAddForNextWidget(gui, GUI_OPTION.Layout_NoLayouting)
      -- Background / Spell type border
      GuiImage(gui, new_id(), x, y, item_bg_icon, 1, background_scale, background_scale)
      _, _, _, last_ac_x, last_ac_y, last_ac_width, last_ac_height = GuiGetPreviousWidgetInfo(gui)
      local _, _, _, x, y, w, h = GuiGetPreviousWidgetInfo(gui)
      GuiOptionsAddForNextWidget(gui, GUI_OPTION.Layout_NoLayouting)
      -- Spell icon
      GuiZSetForNextWidget(gui, z - 1)
      GuiImage(gui, new_id(), x + 2, y + 2, (spell_lookup[spell] and spell_lookup[spell].icon) or "data/ui_gfx/gun_actions/unidentified.png", 1, always_cast_spell_icon_scale, always_cast_spell_icon_scale)
    end
    GuiLayoutEnd(gui)
  end
  -- /Always casts
  -- Spells
  local spell_icon_scale = 0.70066976733398
  local background_scale = 0.76863774490356
  GuiLayoutBeginHorizontal(gui, last_icon_x, last_icon_y + last_icon_height + 7 + add_some + 0.05, true)
  local row = 0
  for i=1, wand.props.capacity do
    GuiZSetForNextWidget(gui, z - 1)
    GuiImage(gui, new_id(), -0.3, -0.4, "data/ui_gfx/inventory/inventory_box.png", 0.95, background_scale, background_scale)
    update_bounds()
    local _, _, _, x, y = GuiGetPreviousWidgetInfo(gui)
    x = x + 0.32479339599609
    y = y + 0.4
    local item_bg_icon = get_spell_bg(wand.spells[i])
    GuiZSetForNextWidget(gui, z - 1.5)
    GuiOptionsAddForNextWidget(gui, GUI_OPTION.Layout_NoLayouting)
    if not wand.spells[i] or wand.spells[i] == "" then
      -- Render an invisible (alpha = 0.0001) item just so it counts for the auto-layout
      GuiImage(gui, new_id(), x - 2, y - 2, item_bg_icon, 0.0001, background_scale, background_scale)
    else
      -- Background / Spell type border
      GuiImage(gui, new_id(), x - 2, y - 2, item_bg_icon, 0.75, background_scale, background_scale)
      GuiZSetForNextWidget(gui, z - 2)
      GuiOptionsAddForNextWidget(gui, GUI_OPTION.Layout_NoLayouting)
      GuiImage(gui, new_id(), x + 0.11, y, (spell_lookup[wand.spells[i]] and spell_lookup[wand.spells[i]].icon) or "data/ui_gfx/gun_actions/unidentified.png", 0.8, spell_icon_scale, spell_icon_scale)
    end
    -- Start a new row after 10 spells
    if i % 10 == 0 then
      row = row + 1
      GuiLayoutEnd(gui)
      _, _, _, _, last_icon_y, last_icon_width, last_icon_height = GuiGetPreviousWidgetInfo(gui)
      GuiLayoutBeginHorizontal(gui, last_icon_x, y + 14.00, true)
    end
  end
  GuiLayoutEnd(gui)
  local wand_sprite = wand.sprite_image_file
  if wand_sprite and wand_sprite ~= "" then
    -- Render wand sprite centered in the space on the right
    local wand_sprite_width, wand_sprite_height = GuiGetImageDimensions(gui, wand.sprite_image_file, 2)
    GuiOptionsAddForNextWidget(gui, GUI_OPTION.Layout_NoLayouting)
    local horizontal_space = right - most_right_text_x
    local vertical_space = spread_text_y + spread_text_height - no_text_y
    horizontal_space = math.max(horizontal_space, wand_sprite_height + 6)
    local wand_sprite_place_center_x = most_right_text_x + horizontal_space / 2
    local wand_sprite_place_center_y = no_text_y + vertical_space / 2
    local wand_sprite_x = wand_sprite_place_center_x - wand_sprite_height / 2
    local wand_sprite_y = wand_sprite_place_center_y + wand_sprite_width / 2
    GuiZSetForNextWidget(gui, z - 1)
    GuiImage(gui, new_id(), wand_sprite_x, wand_sprite_y, wand.sprite_image_file, 1, 2, 2, -math.rad(90))
    update_bounds(-90)
  end

  GuiZSetForNextWidget(gui, z)
  GuiImageNinePiece(gui, new_id(), origin_x - 5, origin_y - 5, right - (origin_x - 5) + 5,  bottom - (origin_y - 5) + 5)
  GuiIdPop(gui)
end

local function refresh_wand_if_in_inventory(wand_id)
  -- Refresh the wand if it's being held by something
  local parent = EntityGetRootEntity(wand_id)
  local inventory2_comp = EntityGetFirstComponentIncludingDisabled(parent, "Inventory2Component")
  if inventory2_comp then
    ComponentSetValue2(inventory2_comp, "mForceRefresh", true)
    ComponentSetValue2(inventory2_comp, "mActualActiveItem", 0)
  end
end

local function add_spell_at_pos(wand, action_id, pos)
  local spells_on_wand = wand:GetSpells()
  -- Check if there's space for one more spell
  if wand.capacity == #spells_on_wand then
    return false
  end
  -- Check if there's already a spell at the desired position
  for i, spell in ipairs(spells_on_wand) do
    if spell.inventory_x + 1 == pos then
      return false
    end
  end
  local action_entity_id = CreateItemActionEntity(action_id)
  EntityAddChild(wand.entity_id, action_entity_id)
  EntitySetComponentsWithTagEnabled(action_entity_id, "enabled_in_world", false)
  local item_component = EntityGetFirstComponentIncludingDisabled(action_entity_id, "ItemComponent")
  ComponentSetValue2(item_component, "inventory_slot", pos-1, 0)
  return true
end

-- ##########################
-- ####    UTILS END     ####
-- ##########################

local wand = {}
-- Setter
wand.__newindex = function(table, key, value)
  if rawget(table, "_protected")[key] ~= nil then
    error("Cannot set protected property '" .. key .. "'")
  end
  table:SetProperties({ [key] = value })
end
-- Getter
wand.__index = function(table, key)
  if type(rawget(wand, key)) == "function" then
    return rawget(wand, key)
  end
  if rawget(table, "_protected")[key] ~= nil then
    return rawget(table, "_protected")[key]
  end
  return table:GetProperties({ key })[key]
end

function wand:new(from, rng_seed_x, rng_seed_y)
  -- 'protected' should not be accessed by end users!
  local protected = {}
  local o = {
    _protected = protected
  }
  setmetatable(o, self)
  if type(from) == "table" or from == nil then
    -- Just load some existing wand that we alter later instead of creating one from scratch
    protected.entity_id = EntityLoad("data/entities/items/wand_level_04.xml", rng_seed_x or 0, rng_seed_y or 0)
    protected.ability_component = EntityGetFirstComponentIncludingDisabled(protected.entity_id, "AbilityComponent")
    protected.item_component = EntityGetFirstComponentIncludingDisabled(protected.entity_id, "ItemComponent")
    -- Copy all validated props over or initialize with defaults
    local props = from or {}
    validate_wand_properties(props)
    o:SetProperties(props)
    o:RemoveSpells()
    o:DetachSpells()
  elseif tonumber(from) or type(from) == "number" then
    -- Wrap an existing wand
    protected.entity_id = from
    protected.ability_component = EntityGetFirstComponentIncludingDisabled(protected.entity_id, "AbilityComponent")
    protected.item_component = EntityGetFirstComponentIncludingDisabled(protected.entity_id, "ItemComponent")
  else
    if starts_with(from, "EZW") then
      local values = deserialize(from)
      protected.entity_id = EntityLoad("data/entities/items/wand_level_04.xml", rng_seed_x or 0, rng_seed_y or 0)
      protected.ability_component = EntityGetFirstComponentIncludingDisabled(protected.entity_id, "AbilityComponent")
      protected.item_component = EntityGetFirstComponentIncludingDisabled(protected.entity_id, "ItemComponent")
      validate_wand_properties(values.props)
      o:SetProperties(values.props)
      o:RemoveSpells()
      o:DetachSpells()
      -- Filter spells whose ID no longer exist (for instance when a modded spellpack was disabled)
      values.spells = filter_spells(values.spells)
      values.always_cast_spells = filter_spells(values.always_cast_spells)
      for i, action_id in ipairs(values.spells) do
        if action_id ~= "" then
          add_spell_at_pos(o, action_id, i)
        end
      end
      o:AttachSpells(values.always_cast_spells)
      o:SetSprite(values.sprite_image_file, values.offset_x, values.offset_y, values.tip_x, values.tip_y)
      o:SetName(values.name, values.show_name_in_ui)
    -- Load a wand by xml
    elseif ends_with(from, ".xml") then
      local x, y = GameGetCameraPos()
      protected.entity_id = EntityLoad(from, rng_seed_x or x, rng_seed_y or y)
      protected.ability_component = EntityGetFirstComponentIncludingDisabled(protected.entity_id, "AbilityComponent")
      protected.item_component = EntityGetFirstComponentIncludingDisabled(protected.entity_id, "ItemComponent")
    else
      error("Wrong format for wand creation.", 2)
    end
  end

  if not entity_is_wand(protected.entity_id) then
    error("Loaded entity is not a wand.", 2)
  end

  return o
end

local variable_mappings = {
  shuffle = { target = "gun_config", name = "shuffle_deck_when_empty" },
  spellsPerCast = { target = "gun_config", name="actions_per_round"},
  castDelay = { target = "gunaction_config", name="fire_rate_wait"},
  currentCastDelay = { target = "ability_component", name="mNextFrameUsable"},
  rechargeTime = { target = "gun_config", name="reload_time"},
  currentRechargeTime = { target = "ability_component", name="mReloadNextFrameUsable"},
  manaMax = { target = "ability_component", name="mana_max"},
  mana = { target = "ability_component", name="mana"},
  manaChargeSpeed = { target = "ability_component", name="mana_charge_speed"},
  capacity = { target = "gun_config", name="deck_capacity"},
  spread = { target = "gunaction_config", name="spread_degrees"},
  speedMultiplier = { target = "gunaction_config", name="speed_multiplier"},
}

-- Sets the actual property on the corresponding component/object
function wand:_SetProperty(key, value)
  local mapped_key = variable_mappings[key].name
  local target_setters = {
    ability_component = function(key, value)
      if key == "mNextFrameUsable" then
        ComponentSetValue2(self.ability_component, key, GameGetFrameNum() + value)
        ComponentSetValue2(self.ability_component, "mCastDelayStartFrame", GameGetFrameNum())
      elseif key == "mReloadNextFrameUsable" then
        ComponentSetValue2(self.ability_component, key, GameGetFrameNum() + value)
        ComponentSetValue2(self.ability_component, "mReloadFramesLeft", value)
        ComponentSetValue2(self.ability_component, "reload_time_frames", value)
      else
        ComponentSetValue2(self.ability_component, key, value)
      end
    end,
    gunaction_config = function(key, value)
      ComponentObjectSetValue2(self.ability_component, "gunaction_config", key, value)
    end,
    gun_config = function(key, value)
      ComponentObjectSetValue2(self.ability_component, "gun_config", key, value)
    end,
  }
  -- We need a special rule for capacity, since always cast spells count towards capacity, but not in the UI...
  if key == "capacity" then
    local spells, attached_spells = self:GetSpells()
    -- If capacity is getting reduced, remove any spells that don't fit anymore
    local spells_to_remove = {}
    for i=#spells, value+1, -1 do
      table.insert(spells_to_remove, { spells[i].action_id, 1 })
    end
    if #spells_to_remove > 0 then
      self:RemoveSpells(spells_to_remove)
    end
    value = value + #attached_spells
  end
  target_setters[variable_mappings[key].target](mapped_key, value)
end
-- Retrieves the actual property from the component or object
function wand:_GetProperty(key)
  if not variable_mappings[key] then
    error(("EZWand has no property '%s'"):format(key), 4)
  end
  local mapped_key = variable_mappings[key].name
  local target_getters = {
    ability_component = function(key)
      if key == "mNextFrameUsable" or key == "mReloadNextFrameUsable" then
        return (math.max(0, ComponentGetValue2(self.ability_component, key) - GameGetFrameNum()))
      else
        return ComponentGetValue2(self.ability_component, key)
      end
    end,
    gunaction_config = function(key)
      return ComponentObjectGetValue2(self.ability_component, "gunaction_config", key)
    end,
    gun_config = function(key)
      return ComponentObjectGetValue2(self.ability_component, "gun_config", key)
    end,
  }
  local result = target_getters[variable_mappings[key].target](mapped_key)
  -- We need a special rule for capacity, since always cast spells count towards capacity, but not in the UI...
  if key == "capacity" then
    result = result - select(2, self:GetSpellsCount())
  end
  return result
end

function wand:SetProperties(key_values)
  for k,v in pairs(key_values) do
    validate_property(k, v)
    self:_SetProperty(k, v)
  end
end

function wand:GetProperties(keys)
  -- Return all properties when empty
  if keys == nil then
    keys = {}
    for k,v in pairs(wand_props) do
      table.insert(keys, k)
    end
  end
  local result = {}
  for i,key in ipairs(keys) do
    result[key] = self:_GetProperty(key)
  end
  return result
end
-- For making the interface nicer, this allows us to use this one function here for
function wand:_AddSpells(spells, attach)
  -- Check if capacity is sufficient
  local count = 0
  for i, v in ipairs(spells) do
    count = count + v[2]
  end
  local spells_on_wand = self:GetSpells()
  local positions = {}
  for i, v in ipairs(spells_on_wand) do
    positions[v.inventory_x] = true
  end

  if not attach and #spells_on_wand + count > self.capacity then
    error(string.format("Wand capacity (%d/%d) cannot fit %d more spells. ", #spells_on_wand, self.capacity, count), 3)
  end
  local current_position = 0
  for i,spell in ipairs(spells) do
    for i2=1, spell[2] do
      if not attach then
        local action_entity_id = CreateItemActionEntity(spell[1])
        EntityAddChild(self.entity_id, action_entity_id)
        EntitySetComponentsWithTagEnabled(action_entity_id, "enabled_in_world", false)
        local item_component = EntityGetFirstComponentIncludingDisabled(action_entity_id, "ItemComponent")
        while positions[current_position] do
          current_position = current_position + 1
        end
        positions[current_position] = true
        ComponentSetValue2(item_component, "inventory_slot", current_position, 0)
      else
        AddGunActionPermanent(self.entity_id, spell[1])
      end
    end
  end
  refresh_wand_if_in_inventory(self.entity_id)
end

function extract_spells_from_vararg(...)
  local spells = {}
  local spell_args = select("#", ...) == 1 and type(...) == "table" and ... or {...}
  local i = 1
  while i <= #spell_args do
    if type(spell_args[i]) == "table" then
      -- Check for this syntax: { "BOMB", 1 }
      if type(spell_args[i][1]) ~= "string" or type(spell_args[i][2]) ~= "number" then
        error("Wrong argument format at index " .. i .. ". Expected format for multiple spells shortcut: { \"BOMB\", 3 }", 3)
      else
        table.insert(spells, spell_args[i])
      end
    elseif type(spell_args[i]) == "string" then
      local amount = spell_args[i+1]
      if type(amount) ~= "number" then
        amount = 1
        table.insert(spells, { spell_args[i], amount })
      else
        table.insert(spells, { spell_args[i], amount })
        i = i + 1
      end
    else
      error("Wrong argument format.", 3)
    end
    i = i + 1
  end
  return spells
end
-- Input can be a table of action_ids, or multiple arguments
-- e.g.:
-- AddSpells("BLACK_HOLE")
-- AddSpells("BLACK_HOLE", "BLACK_HOLE", "BLACK_HOLE")
-- AddSpells({"BLACK_HOLE", "BLACK_HOLE"})
-- To add multiple spells you can also use this shortcut:
-- AddSpells("BLACK_HOLE", {"BOMB", 5}) this will add 1 blackhole followed by 5 bombs
function wand:AddSpells(...)
  local spells = extract_spells_from_vararg(...)
  self:_AddSpells(spells, false)
end
-- Same as AddSpells but permanently attach the spells
function wand:AttachSpells(...)
  local spells = extract_spells_from_vararg(...)
  self:_AddSpells(spells, true)
end
-- Returns the amount of slots on a wand that are not occupied by a spell
function wand:GetFreeSlotsCount()
  return self.capacity - self:GetSpellsCount()
end
-- Returns: spells_count, always_cast_spells_count
function wand:GetSpellsCount()
    local children = EntityGetAllChildren(self.entity_id)
  if children == nil then
    return 0, 0
  end
  -- Count the number of always cast spells
  local always_cast_spells_count = 0
  for i,spell in ipairs(children) do
    local item_component = EntityGetFirstComponentIncludingDisabled(spell, "ItemComponent")
    if item_component ~= nil and ComponentGetValue2(item_component, "permanently_attached") == true then
      always_cast_spells_count = always_cast_spells_count + 1
    end
  end

    return #children - always_cast_spells_count, always_cast_spells_count
end
-- Returns two values:
-- 1: table of spells with each entry having the format { action_id = "BLACK_HOLE", inventory_x = 1, entity_id = <action_entity_id> }
-- 2: table of attached spells with the same format
-- inventory_x should give the position in the wand slots, 1 = first up to num_slots
-- inventory_x is not working yet
function wand:GetSpells()
    local spells = {}
    local always_cast_spells = {}
    local children = EntityGetAllChildren(self.entity_id)
  if children == nil then
    return spells, always_cast_spells
  end
    for _, spell in ipairs(children) do
        local action_id = nil
        local permanent = false
    local item_action_component = EntityGetFirstComponentIncludingDisabled(spell, "ItemActionComponent")
    if item_action_component then
      action_id = ComponentGetValue2(item_action_component, "action_id")
    end
    local inventory_x, inventory_y = -1, -1
    local item_component = EntityGetFirstComponentIncludingDisabled(spell, "ItemComponent")
    if item_component then
      permanent = ComponentGetValue2(item_component, "permanently_attached")
      inventory_x, inventory_y = ComponentGetValue2(item_component, "inventory_slot")
    end
    if action_id then
            if permanent == true then
                table.insert(always_cast_spells, { action_id = action_id, entity_id = spell, inventory_x = inventory_x, inventory_y = inventory_y })
            else
                table.insert(spells, { action_id = action_id, entity_id = spell, inventory_x = inventory_x, inventory_y = inventory_y })
            end
        end
  end

  local function assign_inventory_x(t)
    local a = {}
    for i, v in ipairs(t) do
      if v.inventory_x > 0 then
        a[v.inventory_x+1] = v
      end
    end
    local inventory_x = 1
    for i, v in ipairs(t) do
      if v.inventory_x == 0 then
        while a[inventory_x] do
          inventory_x = inventory_x + 1
        end
        v.inventory_x = inventory_x-1
        a[inventory_x] = v
      end
    end
    for i = #t, 1, -1 do
      if not t[i].inventory_x then
        table.remove(t, i)
      end
    end
  end
  -- When a wand is spawned its spell's inventory_x is always set to 0, only once the inventory is opened
  -- is inventory_x assigned correctly to all spells, so to fake that we go through all the spells manually
  -- and assign inventory_x to either what it was set as or by the order the entities appear on the wand
  assign_inventory_x(spells)
  table.sort(spells, function(a, b) return a.inventory_x < b.inventory_x end)
    return spells, always_cast_spells
end

function wand:_RemoveSpells(spells_to_remove, detach)
    local spells, attached_spells = self:GetSpells()
  local which = detach and attached_spells or spells
  local spells_to_remove_remaining = {}
  for _, spell in ipairs(spells_to_remove) do
    spells_to_remove_remaining[spell[1]] = (spells_to_remove_remaining[spell[1]] or 0) + spell[2]
  end
  for i, v in ipairs(which) do
    if #spells_to_remove == 0 or spells_to_remove_remaining[v.action_id] and spells_to_remove_remaining[v.action_id] ~= 0 then
      if #spells_to_remove > 0 then
        spells_to_remove_remaining[v.action_id] = spells_to_remove_remaining[v.action_id] - 1
      end
      -- This needs to happen because EntityKill takes one frame to take effect or something
      EntityRemoveFromParent(v.entity_id)
      EntityKill(v.entity_id)
      if detach then
        self.capacity = self.capacity - 1
      end
    end
  end
  refresh_wand_if_in_inventory(self.entity_id)
end
-- action_ids = {"BLACK_HOLE", "GRENADE"} remove all spells of those types
-- If action_ids is empty, remove all spells
-- If entry is in the form of {"BLACK_HOLE", 2}, only remove 2 instances of black hole
function wand:RemoveSpells(...)
  local spells = extract_spells_from_vararg(...)
  self:_RemoveSpells(spells, false)
end
function wand:DetachSpells(...)
  local spells = extract_spells_from_vararg(...)
  self:_RemoveSpells(spells, true)
end

function wand:RemoveSpellAtIndex(index)
  if index+1 > self.capacity then
    return false, "index is bigger than capacity"
  end
  local spells = self:GetSpells()
  for i, spell in ipairs(spells) do
    if spell.inventory_x == index then
      -- This needs to happen because EntityKill takes one frame to take effect or something
      EntityRemoveFromParent(spell.entity_id)
      EntityKill(spell.entity_id)
      return true
    end
  end
  return false, "index at " .. index .. " does not contain a spell"
end

-- Make it impossible to edit the wand
-- freeze_wand prevents spells from being added to the wand or moved
-- freeze_spells prevents the spells from being removed
function wand:SetFrozen(freeze_wand, freeze_spells)
  local item_component = EntityGetFirstComponentIncludingDisabled(self.entity_id, "ItemComponent")
  ComponentSetValue2(item_component, "is_frozen", freeze_wand)
  local spells = self:GetSpells()
  for i, spell in ipairs(spells) do
    local item_component = EntityGetFirstComponentIncludingDisabled(spell.entity_id, "ItemComponent")
    ComponentSetValue2(item_component, "is_frozen", freeze_spells)
  end
end

function wand:SetSprite(item_file, offset_x, offset_y, tip_x, tip_y)
    if self.ability_component then
    ComponentSetValue2(self.ability_component, "sprite_file", item_file)
    end
  local sprite_comp = EntityGetFirstComponentIncludingDisabled(self.entity_id, "SpriteComponent", "item")
  if sprite_comp then
    ComponentSetValue2(sprite_comp, "image_file", item_file)
    ComponentSetValue2(sprite_comp, "offset_x", offset_x)
    ComponentSetValue2(sprite_comp, "offset_y", offset_y)
    EntityRefreshSprite(self.entity_id, sprite_comp)
    end
    local hotspot_comp = EntityGetFirstComponentIncludingDisabled(self.entity_id, "HotspotComponent", "shoot_pos")
  if hotspot_comp then
    ComponentSetValue2(hotspot_comp, "offset", tip_x, tip_y)
    end
end

function wand:GetSprite()
  local sprite_file, offset_x, offset_y, tip_x, tip_y = "", 0, 0, 0, 0
    if self.ability_component then
        sprite_file = ComponentGetValue2(self.ability_component, "sprite_file")
    end
    local sprite_comp = EntityGetFirstComponentIncludingDisabled(self.entity_id, "SpriteComponent", "item")
    if sprite_comp then
    if sprite_file == "" then
      sprite_file = ComponentGetValue2(sprite_comp, "image_file")
    end
    offset_x = ComponentGetValue2(sprite_comp, "offset_x")
    offset_y = ComponentGetValue2(sprite_comp, "offset_y")
    end
    local hotspot_comp = EntityGetFirstComponentIncludingDisabled(self.entity_id, "HotspotComponent", "shoot_pos")
  if hotspot_comp then
    tip_x, tip_y = ComponentGetValue2(hotspot_comp, "offset")
  end
  return sprite_file, offset_x, offset_y, tip_x, tip_y
end

function wand:Clone()
  local new_wand = wand:new(self:GetProperties())
  local spells, attached_spells = self:GetSpells()
  for k, v in pairs(spells) do
    new_wand:AddSpells{v.action_id}
  end
  for k, v in pairs(attached_spells) do
    new_wand:AttachSpells{v.action_id}
  end
  -- TODO: Make this work if sprite_file is an xml
  new_wand:SetSprite(self:GetSprite())
  return new_wand
end

--[[
  These are pulled from data/scripts/gun/procedural/gun_procedural.lua
  because dofiling that file overwrites the init_total_prob function,
  which ruins things in biome scripts
]]
function WandDiff( gun, wand )
    local score = 0
    score = score + ( math.abs( gun.fire_rate_wait - wand.fire_rate_wait ) * 2 )
    score = score + ( math.abs( gun.actions_per_round - wand.actions_per_round ) * 20 )
    score = score + ( math.abs( gun.shuffle_deck_when_empty - wand.shuffle_deck_when_empty ) * 30 )
    score = score + ( math.abs( gun.deck_capacity - wand.deck_capacity ) * 5 )
    score = score + math.abs( gun.spread_degrees - wand.spread_degrees )
    score = score + math.abs( gun.reload_time - wand.reload_time )
    return score
end

function GetWand( gun )
    local best_wand = nil
    local best_score = 1000
    local gun_in_wand_space = {}

    gun_in_wand_space.fire_rate_wait = clamp(((gun["fire_rate_wait"] + 5) / 7)-1, 0, 4)
    gun_in_wand_space.actions_per_round = clamp(gun["actions_per_round"]-1,0,2)
    gun_in_wand_space.shuffle_deck_when_empty = clamp(gun["shuffle_deck_when_empty"], 0, 1)
    gun_in_wand_space.deck_capacity = clamp( (gun["deck_capacity"]-3)/3, 0, 7 ) -- TODO
    gun_in_wand_space.spread_degrees = clamp( ((gun["spread_degrees"] + 5 ) / 5 ) - 1, 0, 2 )
    gun_in_wand_space.reload_time = clamp( ((gun["reload_time"]+5)/25)-1, 0, 2 )

    for k,wand in pairs(wands) do
        local score = WandDiff( gun_in_wand_space, wand )
        if( score <= best_score ) then
            best_wand = wand
            best_score = score
            -- just randomly return one of them...
            if( score == 0 and Random(0,100) < 33 ) then
                return best_wand
            end
        end
    end
    return best_wand
end
--[[ /data/scripts/gun/procedural/gun_procedural.lua ]]

-- Applies an appropriate Sprite using the games own algorithm
function wand:UpdateSprite()
  local gun = {
    fire_rate_wait = self.castDelay,
    actions_per_round = self.spellsPerCast,
    shuffle_deck_when_empty = self.shuffle and 1 or 0,
    deck_capacity = self.capacity,
    spread_degrees = self.spread,
    reload_time = self.rechargeTime,
  }
  local sprite_data = GetWand(gun)
  self:SetSprite(sprite_data.file, sprite_data.grip_x, sprite_data.grip_y,
    (sprite_data.tip_x - sprite_data.grip_x),
    (sprite_data.tip_y - sprite_data.grip_y))
end

function wand:SetName(name, show_in_ui)
  if show_in_ui == nil then
    show_in_ui = true
  end
  show_in_ui = not not show_in_ui
  local item_comp = self.item_component
  if item_comp then
    ComponentSetValue2(item_comp, "item_name", tostring(name))
    ComponentSetValue2(item_comp, "always_use_item_name_in_ui", show_in_ui)
  end
end

-- returns name, show_in_ui
function wand:GetName()
  local item_comp = self.item_component
  if item_comp then
    local name = ComponentGetValue2(item_comp, "item_name")
    local show_in_ui = ComponentGetValue2(item_comp, "always_use_item_name_in_ui")
    return name, show_in_ui
  end
  error("No item component found", 2)
end

function wand:PickUp(entity)
    local item_component = EntityGetFirstComponentIncludingDisabled(self.entity_id, "ItemComponent")
    local preferred_inv = "QUICK"
    if item_component then
      ComponentSetValue2(item_component, "has_been_picked_by_player", true)
      preferred_inv = ComponentGetValue2(item_component, "preferred_inventory")
    end

    local entity_children = EntityGetAllChildren(entity) or {}

    for key, child in pairs( entity_children ) do
      if EntityGetName( child ) == "inventory_"..string.lower(preferred_inv) then
        EntityAddChild( child, self.entity_id)
      end
    end

    EntitySetComponentsWithTagEnabled( self.entity_id, "enabled_in_world", false )
    EntitySetComponentsWithTagEnabled( self.entity_id, "enabled_in_hand", false )
    EntitySetComponentsWithTagEnabled( self.entity_id, "enabled_in_inventory", true )

    local wand_children = EntityGetAllChildren(self.entity_id) or {}

    for k, v in ipairs(wand_children)do
      EntitySetComponentsWithTagEnabled( self.entity_id, "enabled_in_world", false )
    end

    local sprite_particle_emitter_comp = EntityGetFirstComponentIncludingDisabled(self.entity_id, "SpriteParticleEmitterComponent")
    if sprite_particle_emitter_comp ~= nil then
        EntitySetComponentIsEnabled(self.entity_id, sprite_particle_emitter_comp, false)
    end

end

function wand:PlaceAt(x, y)
    EntitySetComponentIsEnabled(self.entity_id, self.ability_component, true)
    local hotspot_comp = EntityGetFirstComponentIncludingDisabled(self.entity_id, "HotspotComponent")
    EntitySetComponentIsEnabled(self.entity_id, hotspot_comp, true)
  local item_component = EntityGetFirstComponentIncludingDisabled(self.entity_id, "ItemComponent")
    EntitySetComponentIsEnabled(self.entity_id, item_component, true)
    local sprite_component = EntityGetFirstComponentIncludingDisabled(self.entity_id, "SpriteComponent")
    EntitySetComponentIsEnabled(self.entity_id, sprite_component, true)
  local light_component = EntityGetFirstComponentIncludingDisabled(self.entity_id, "LightComponent")
  EntitySetComponentIsEnabled(self.entity_id, light_component, true)

  ComponentSetValue(item_component, "has_been_picked_by_player", "0")
  ComponentSetValue(item_component, "play_hover_animation", "1")
  ComponentSetValueVector2(item_component, "spawn_pos", x, y)

    local lua_comp = EntityGetFirstComponentIncludingDisabled(self.entity_id, "LuaComponent")
    EntitySetComponentIsEnabled(self.entity_id, lua_comp, true)
    local simple_physics_component = EntityGetFirstComponentIncludingDisabled(self.entity_id, "SimplePhysicsComponent")
  EntitySetComponentIsEnabled(self.entity_id, simple_physics_component, false)
    -- Does this wand have a ray particle effect? Most do, except the starter wands
    local sprite_particle_emitter_comp = EntityGetFirstComponentIncludingDisabled(self.entity_id, "SpriteParticleEmitterComponent")
    if sprite_particle_emitter_comp ~= nil then
        EntitySetComponentIsEnabled(self.entity_id, sprite_particle_emitter_comp, true)
    else
        -- TODO: As soon as there's some way to clone Components or Transplant/Remove+Add to another Entity, copy
        -- the SpriteParticleEmitterComponent of entities/base_wand.xml
  end
end

function wand:PutInPlayersInventory()
  local inventory_id = EntityGetWithName("inventory_quick")
  -- Get number of wands currently already in inventory
  local count = 0
  local inventory_items = EntityGetAllChildren(inventory_id)
  if inventory_items then
    for i,v in ipairs(inventory_items) do
      if entity_is_wand(v) then
        count = count + 1
      end
    end
  end
  local players = EntityGetWithTag("player_unit")
  if count < 4 and #players > 0 then
    local item_component = EntityGetFirstComponentIncludingDisabled(self.entity_id, "ItemComponent")
    if item_component then
      ComponentSetValue2(item_component, "has_been_picked_by_player", true)
    end
    GamePickUpInventoryItem(players[1], self.entity_id, false)
  else
    error("Cannot add wand to players inventory, it's already full.", 2)
  end
end

-- Turns the wand properties etc into a string
-- Output string looks like:
-- EZWv(version);shuffle[1|0];spellsPerCast;castDelay;rechargeTime;manaMax;mana;manaChargeSpeed;capacity;spread;speedMultiplier;
-- SPELL_ONE,SPELL_TWO;ALWAYS_CAST_ONE,ALWAYS_CAST_TWO;sprite.png;offset_x;offset_y;tip_x;tip_y;name;show_name
function wand:Serialize()
  local spells_string = ""
  local always_casts_string = ""
  local spells, always_casts = self:GetSpells()
  local slots = {}
  for i, spell in ipairs(spells) do
    slots[spell.inventory_x+1] = spell
  end
  for i=1, self.capacity do
    spells_string = spells_string .. (i == 1 and "" or ",") .. (slots[i] and slots[i].action_id or "")
  end
  for i, spell in ipairs(always_casts) do
    always_casts_string = always_casts_string .. (i == 1 and "" or ",") .. spell.action_id
  end

  local sprite_image_file, offset_x, offset_y, tip_x, tip_y = self:GetSprite()

  -- Add a workaround for the starter wands which are the only ones with an xml sprite
  -- Modded wands which use xmls won't work sadly
  if sprite_image_file == "data/items_gfx/handgun.xml" then
    sprite_image_file = "data/items_gfx/handgun.png"
    offset_x = 4
    offset_y = 3.5
  end
  if sprite_image_file == "data/items_gfx/bomb_wand.xml" then
    sprite_image_file = "data/items_gfx/bomb_wand.png"
    offset_x = 4
    offset_y = 3.5
  end

  local name, show_name = self:GetName()
  local serialize_version = "2"
  return ("EZWv%s;%d;%d;%d;%d;%d;%d;%d;%d;%d;%d;%s;%s;%s;%d;%d;%d;%d;%s;%d"):format(
    serialize_version,
    self.shuffle and 1 or 0,
    self.spellsPerCast,
    self.castDelay,
    self.rechargeTime,
    self.manaMax,
    self.mana,
    self.manaChargeSpeed,
    self.capacity,
    self.spread,
    self.speedMultiplier,
    spells_string == "" and "-" or spells_string,
    always_casts_string == "" and "-" or always_casts_string,
    sprite_image_file, offset_x, offset_y, tip_x, tip_y, name, show_name and 1 or 0
  )
end

local function get_held_wand()
    local player = EntityGetWithTag("player_unit")[1]
  if player then
    local inventory2_comp = EntityGetFirstComponentIncludingDisabled(player, "Inventory2Component")
    local active_item = ComponentGetValue2(inventory2_comp, "mActiveItem")
    return entity_is_wand(active_item) and wand:new(active_item)
  end
end

function wand:RenderTooltip(origin_x, origin_y, gui_, z)
  local success, error_msg = pcall(render_tooltip, origin_x, origin_y, deserialize(self:Serialize()), gui_, z)
  if not success then
    error(error_msg, 2)
  end
end

local virtual_wand = {}
local virtual_wand_privates = setmetatable({}, { __mode = "k" })
local virtual_wand_props = {
  visible = {
    get = function(self)
      return virtual_wand_privates[self].visible
    end,
    set = function(self, value)
      if type(value) ~= "boolean" then
        error("visible must be a boolean", 100)
      end
      local entity_id = self.wand.entity_id
      for i, comp in ipairs(EntityGetComponentIncludingDisabled(entity_id, "SpriteComponent") or {}) do
        -- This makes the inventory component enable/disable the component, can't use EntitySetComponentIsEnabled
        -- because that gets immediately overridden by the inventory system
        if value then
          ComponentAddTag(comp, "enabled_in_hand")
        else
          ComponentRemoveTag(comp, "enabled_in_hand")
        end
      end
      if self.hotspot_comp then
        ComponentSetValue2(self.hotspot_comp, "offset", value and self.shoot_offset_x or 0, value and self.shoot_offset_y or 0)
      end
      virtual_wand_privates[self].visible = value
    end
  },
  rotation = {
    get = function(self)
      return select(3, EntityGetTransform(self.wand.entity_id))
    end,
    set = function(self, value)
      if type(value) ~= "number" then
        error("rotation must be a number", 100)
      end
      local x, y = EntityGetTransform(self.entity_id)
      local vx = math.cos(value) * 50
      local vy = math.sin(value) * 50
      ComponentSetValue2(self.controls_comp, "mAimingVector", vx, vy)
      ComponentSetValue2(self.controls_comp, "mMousePosition", x + vx, y + vy)
    end
  },
  herd = {
    get = function(self)
      local genome_data_comp = EntityGetFirstComponentIncludingDisabled(self.entity_id, "GenomeDataComponent")
      if genome_data_comp then
        return HerdIdToString(ComponentGetValue2(genome_data_comp, "herd_id"))
      end
    end,
    set = function(self, value)
      if value == nil then
        return
      end
      if type(value) ~= "string" then
        error("herd must be a string", 100)
      end
      local genome_data_comp = EntityGetFirstComponentIncludingDisabled(self.entity_id, "GenomeDataComponent")
      if not genome_data_comp then
        genome_data_comp = EntityAddComponent2(self.entity_id, "GenomeDataComponent")
      end
      ComponentSetValue2(genome_data_comp, "herd_id", StringToHerdId(value))
    end
  },
  exists = {
    get = function(self)
      return EntityGetIsAlive(self.entity_id)
    end
  }
}

function virtual_wand:__index(key)
  if rawget(self, "_protected")[key] ~= nil then
    return rawget(self, "_protected")[key]
  elseif rawget(virtual_wand, key) ~= nil then
    return rawget(virtual_wand, key)
  elseif virtual_wand_props[key] then
    return virtual_wand_props[key].get(self)
  else -- Pass on all accesses to the underlying EZWand object
    if type(self.wand[key]) == "function" then
      return function(...)
        return self.wand[key](self.wand, select(2, ...))
      end
    else
      return self.wand[key]
    end
  end
end

function virtual_wand:__newindex(key, value)
  if rawget(self, "_protected")[key] ~= nil then
    error("Cannot set protected property '" .. key .. "'", 2)
  elseif virtual_wand_props[key] then
    if virtual_wand_props[key].set then
      virtual_wand_props[key].set(self, value)
    else
      error("'" .. key .. "' is read only.", 100)
    end
  else -- Pass on all accesses to the underlying EZWand object
    self.wand[key] = value
  end
end

function virtual_wand:SetPosition(x, y)
  EntitySetTransform(self.entity_id, x, y)
end

function virtual_wand:AimAt(target_x, target_y)
  if self.controls_comp then
    local x, y = EntityGetTransform(self.entity_id)
    local aim_x, aim_y = target_x - x, target_y - y
    ComponentSetValue2(self.controls_comp, "mAimingVector", aim_x, aim_y)
    ComponentSetValue2(self.controls_comp, "mMousePosition", target_x, target_y)
  end
end

function virtual_wand:Shoot(a)
  if a ~= nil then
    error("Shoot() doesn't require parameters, did you mean ShootAt(x, y)?", 100)
  end
  if self.platform_shooter_player_comp then
    ComponentSetValue2(self.platform_shooter_player_comp, "mForceFireOnNextUpdate", true)
  end
end

function virtual_wand:ShootAt(target_x, target_y)
  self:AimAt(target_x, target_y)
  self:Shoot()
end

-- Have to override this function because we need to add 0.5 to y offset so it doesn't jump when turning
function virtual_wand:SetSprite(sprite_image_file, offset_x, offset_y, tip_x, tip_y)
  self.wand:SetSprite(sprite_image_file, offset_x, offset_y + 0.5, tip_x, tip_y)
end

function virtual_wand:UpdateSprite()
  self.wand:UpdateSprite()
  self:SetSprite(self:GetSprite())
end

-- Takes a wand and removes it from an inventory if it is in one
-- and if it was removed, also updates the inventory active item
local function uninventorify(wand_entity_id)
  -- Use for just to avoid a while loop which could lead to an infinite loop
  local parent = EntityGetParent(wand_entity_id)
  for i=1, 20 do
    if parent > 0 then
      local inv_comp = EntityGetFirstComponentIncludingDisabled(parent, "Inventory2Component")
      if inv_comp then
        ComponentSetValue2(inv_comp, "mActualActiveItem", 0)
        ComponentSetValue2(inv_comp, "mActiveItem", 0)
        ComponentSetValue2(inv_comp, "mForceRefresh", true)
        break
      end
    end
    parent = EntityGetParent(parent)
  end
  if EntityGetParent(wand_entity_id) > 0 then
    EntityRemoveFromParent(wand_entity_id)
  end
end

local function give_wand_to_entity(wand_entity_id, entity_id)
  local inv_quick
  for i, child in ipairs(EntityGetAllChildren(entity_id) or {}) do
    if EntityGetName(child) == "inventory_quick" then
      inv_quick = child
      break
    end
  end
  if not inv_quick then
    inv_quick = EntityCreateNew("inventory_quick")
    EntityAddChild(entity_id, inv_quick)
  end
  uninventorify(wand_entity_id)
  EntityAddChild(inv_quick, wand_entity_id)
  local x, y = EntityGetTransform(entity_id)
  EntitySetTransform(wand_entity_id, x, y)
  local ability_comp = EntityGetFirstComponentIncludingDisabled(wand_entity_id, "AbilityComponent")
  if ability_comp then
    -- Make the wand able to be aimed straight up
    ComponentSetValue2(ability_comp, "rotate_in_hand_amount", 1)
  end
  local required_comps = {
    ControlsComponent = { enabled = false },
    CharacterDataComponent = {},
    SpriteAnimatorComponent = {},
    CharacterPlatformingComponent = {
      mouse_look = false,
    },
    Inventory2Component = {
      quick_inventory_slots = 10,
    },
    GunComponent = {},
    -- PlatformShooterPlayerComponent = {},
    GenomeDataComponent = {
      herd_id = StringToHerdId("zombie"),
            food_chain_rank = 0,
            is_predator = true
    },
    AnimalAIComponent = {
      preferred_job="JobDefault",
      attack_ranged_min_distance=0,
      attack_ranged_max_distance=100,
      creature_detection_range_x=300,
      creature_detection_range_y=300,
    },
    -- Not required but makes wands drop from enemies when killed
    ItemPickUpperComponent = {
      is_in_npc = true,
      is_immune_to_kicks = false,
      drop_items_on_death = true,
    }
  }
  for comp_type, properties in pairs(required_comps) do
    local comp = EntityGetFirstComponentIncludingDisabled(entity_id, comp_type)
    if not comp then
      EntityAddComponent2(entity_id, comp_type, properties)
    end
  end
  local hand_hotspot_comp = EntityGetFirstComponentIncludingDisabled(entity_id, "HotspotComponent", "hand")
  if not hand_hotspot_comp then
    hand_hotspot_comp = EntityAddComponent2(entity_id, "HotspotComponent", { _tags = "hand" })
    local hx, hy = EntityGetFirstHitboxCenter(entity_id)
    local dx, dy = hx - x, hy - y
    ComponentSetValue2(hand_hotspot_comp, "offset", dx + 1, dy - 2)
  end
  local item_comp = EntityGetFirstComponentIncludingDisabled(wand_entity_id, "ItemComponent")
  if item_comp then
    ComponentSetValue2(item_comp, "play_hover_animation", false)
  end
end

function wand:GiveTo(entity_id)
  give_wand_to_entity(self.entity_id, entity_id)
end

local function create_holder_entity()
  -- Need to use EntityLoad because entities created with EntityCreateNew don't get saved
  local entity_id = EntityLoad("data/entities/buildings/coop_respawn.xml")
  EntitySetName(entity_id, "EZWand_virtual_wand")
  EntityRemoveTag(entity_id, "coop_respawn")
  -- local entity_id = EntityCreateNew("EZWand_virtual_wand")
  local inv_quick = EntityCreateNew("inventory_quick")
  EntityAddChild(entity_id, inv_quick)
  -- Set scale_x to a tiny number to prevent a 1 pixel x swap on direction swapping
  -- Thanks to @dextercd on Discord for this tip
  EntitySetTransform(entity_id, 0, 0, 0, 0.000001)
  EntityAddComponent2(entity_id, "CharacterDataComponent", {})
  EntityAddComponent2(entity_id, "SpriteAnimatorComponent", {})
  EntityAddComponent2(entity_id, "Inventory2Component", {})
  EntityAddComponent2(entity_id, "GunComponent", {})
  local controls_comp = EntityAddComponent2(entity_id, "ControlsComponent", {
    enabled = false,
  })
  local character_platforming_component = EntityAddComponent2(entity_id, "CharacterPlatformingComponent", {})
  local platform_shooter_player_comp = EntityAddComponent2(entity_id, "PlatformShooterPlayerComponent", {})
  return entity_id, inv_quick, controls_comp, character_platforming_component, platform_shooter_player_comp
end

local function create_virtual_wand(from, rng_seed_x, rng_seed_y)
  local wand = wand:new(from, rng_seed_x, rng_seed_y)
  -- Still not sure how best to implement read only properties
  local protected = { wand = wand }
  local o = setmetatable({ _protected = protected }, virtual_wand)
  virtual_wand_privates[o] = { visible = true }
  SetRandomSeed(rng_seed_x, rng_seed_y)
  -- Update the sprite to add the 0.5 offset
  o:SetSprite(o:GetSprite())
  local wand_entity_id = wand.entity_id
  protected.hotspot_comp = EntityGetFirstComponentIncludingDisabled(wand_entity_id, "HotspotComponent", "shoot_pos")
  if protected.hotspot_comp then
    protected.shoot_offset_x, protected.shoot_offset_y = ComponentGetValue2(protected.hotspot_comp, "offset")
  end
  local holder_entity, inv_quick, controls_comp, character_platforming_component, platform_shooter_player_comp = create_holder_entity()
  protected.entity_id = holder_entity
  uninventorify(wand_entity_id)
  EntityAddChild(inv_quick, wand_entity_id)
  EntityApplyTransform(holder_entity, rng_seed_x, rng_seed_y)
  EntityApplyTransform(wand_entity_id, rng_seed_x, rng_seed_y)
  protected.controls_comp = controls_comp
  protected.character_platforming_component = character_platforming_component
  protected.platform_shooter_player_comp = platform_shooter_player_comp
  return o
end

function wand:Deploy(x, y)
  local root = EntityGetRootEntity(self.entity_id)
  if EntityGetName(root) ~= "EZWand_virtual_wand" then
    return create_virtual_wand(self.entity_id, x, y)
  else
    local protected = { wand = self }
    local o = setmetatable({ _protected = protected }, virtual_wand)
    virtual_wand_privates[o] = { visible = true }
    SetRandomSeed(x, y)
    -- Update the sprite to add the 0.5 offset
    o:SetSprite(o:GetSprite())
    local wand_entity_id = self.entity_id
    protected.hotspot_comp = EntityGetFirstComponentIncludingDisabled(wand_entity_id, "HotspotComponent", "shoot_pos")
    if protected.hotspot_comp then
      protected.shoot_offset_x, protected.shoot_offset_y = ComponentGetValue2(protected.hotspot_comp, "offset")
    end
    protected.entity_id = root
    local holder_entity = protected.entity_id
    EntityApplyTransform(holder_entity, x, y)
    protected.controls_comp = EntityGetFirstComponentIncludingDisabled(holder_entity, "ControlsComponent")
    protected.character_platforming_component = EntityGetFirstComponentIncludingDisabled(holder_entity, "CharacterPlatformingComponent")
    protected.platform_shooter_player_comp = EntityGetFirstComponentIncludingDisabled(holder_entity, "PlatformShooterPlayerComponent")
    return o
  end
end

local function shoot_spell_sequence(sequence, from_x, from_y, target_x, target_y, herd)
  local wand = create_virtual_wand({
    shuffle = false,
    spellsPerCast = 1,
    castDelay = 0,
    rechargeTime = 0,
    manaMax = 10000,
    manaChargeSpeed = 100,
    capacity = #sequence,
    spread = -100
  }, from_x, from_y)
  wand.herd = herd
  wand.visible = false
  wand:AddSpells(sequence)
  wand:ShootAt(target_x, target_y)
  EntityAddComponent2(wand.entity_id, "LifetimeComponent", { lifetime = 1 })
end

return setmetatable({}, {
  __call = function(self, from, rng_seed_x, rng_seed_y)
    return wand:new(from, rng_seed_x, rng_seed_y)
  end,
  __newindex = function(self)
    error("Can't assign to this object.", 2)
  end,
  __index = function(self, key)
    return ({
      Deserialize = deserialize,
      RenderTooltip = render_tooltip,
      IsWand = entity_is_wand,
      GetHeldWand = get_held_wand,
      ShootSpellSequence = shoot_spell_sequence
    })[key]
  end
})