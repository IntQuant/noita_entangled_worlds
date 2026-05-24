local EZWand = dofile_once("mods/quant.ew/files/lib/EZWand.lua")

local gui = GuiCreate()

local module = {}

local INSPECT_DISTANCE = 160
local INSPECT_DISTANCE_SQ = INSPECT_DISTANCE * INSPECT_DISTANCE

local function inspect_key_down()
    local rebind = tonumber(ModSettingGet("quant.ew.rebind_wand_showcase") or 226) or 226
    return rebind ~= 0 and InputIsKeyDown(rebind)
end

local function get_active_wand(player_data)
    if player_data == nil or player_data.entity == nil or not EntityGetIsAlive(player_data.entity) then
        return nil
    end

    local active_item = player_fns.get_active_held_item(player_data.entity)
    if active_item == nil or active_item == 0 or not EZWand.IsWand(active_item) then
        return nil
    end

    return EZWand(active_item)
end

local function nearest_player()
    local my_x, my_y = EntityGetTransform(ctx.my_player.entity)
    local nearest
    local nearest_dist_sq = INSPECT_DISTANCE_SQ

    for peer_id, player_data in pairs(ctx.players) do
        if peer_id ~= ctx.my_id and player_data.entity ~= nil and EntityGetIsAlive(player_data.entity) and not player_data.dc then
            local x, y = EntityGetTransform(player_data.entity)
            local dx = x - my_x
            local dy = y - my_y
            local dist_sq = dx * dx + dy * dy
            if dist_sq <= nearest_dist_sq then
                nearest = player_data
                nearest_dist_sq = dist_sq
            end
        end
    end

    return nearest
end

local function text_with_shadow(x, y, text)
    GuiZSetForNextWidget(gui, -3)
    GuiColorSetForNextWidget(gui, 1, 1, 1, 1)
    GuiText(gui, x, y, text)
    GuiZSetForNextWidget(gui, -2)
    GuiOptionsAddForNextWidget(gui, GUI_OPTION.Layout_NoLayouting)
    GuiColorSetForNextWidget(gui, 0, 0, 0, 0.85)
    local _, _, _, prev_x, prev_y = GuiGetPreviousWidgetInfo(gui)
    GuiText(gui, prev_x, prev_y + 1, text)
end

local function render_wand(label, wand, x, y)
    text_with_shadow(x, y, label)
    local ok, right, bottom = pcall(function()
        return wand:RenderTooltip(x, y + 11, gui, -2)
    end)
    if ok then
        return right, bottom
    end

    text_with_shadow(x, y + 11, "Could not render wand")
    return x + 90, y + 22
end

function module.on_world_update()
    if ctx.is_texting or ctx.is_paused or not inspect_key_down() then
        return
    end

    local target = nearest_player()
    if target == nil then
        return
    end

    local target_wand = get_active_wand(target)
    if target_wand == nil then
        return
    end

    GuiStartFrame(gui)
    GuiZSet(gui, -2)

    local screen_w = GuiGetScreenDimensions(gui)
    local x = 8
    local y = 18
    local target_label = (target.name or "Player") .. "'s wand"
    local right = render_wand(target_label, target_wand, x, y)

    local my_wand = get_active_wand(ctx.my_player)
    if my_wand ~= nil then
        local compare_x = math.max(right + 14, screen_w * 0.48)
        render_wand("Your wand", my_wand, compare_x, y)
    end
end

return module
