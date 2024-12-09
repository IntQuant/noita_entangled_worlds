local module = {}
local gui = GuiCreate()
local rpc = net.new_rpc_namespace()

local text = ""
local enabled = false

rpc.opts_everywhere()
rpc.opts_reliable()

local chatMessages = {}
local maxMessages = 128
local lineHeight = 10
local maxVisibleLines = 15
local maxInputLength = 512
local visibleChars = 85
local currentMessageIndex = 1

local function world2gui( x, y )
    in_camera_ref = in_camera_ref or false

    local gui_n = GuiCreate()
    GuiStartFrame(gui_n)
    local w, h = GuiGetScreenDimensions(gui_n)
    GuiDestroy(gui_n)

    local vres_scaling_factor = w / (MagicNumbersGetValue("VIRTUAL_RESOLUTION_X") + MagicNumbersGetValue("VIRTUAL_RESOLUTION_OFFSET_X"))
    local cam_x, cam_y = GameGetCameraPos()
    x, y = w / 2 + vres_scaling_factor * (x - cam_x), h / 2 + vres_scaling_factor * (y - cam_y)

    return x, y, vres_scaling_factor
end

local function calculateTextWidth(gui, text)
    local width, _ = GuiGetTextDimensions(gui, text)
    return width
end

local function getColorComponents(color)
    local r = math.floor(color / (256 * 256)) % 256
    local g = math.floor(color / 256) % 256
    local b = color % 256
    return r, g, b
end

local function saveMessage(sender, message, color, colorAlt)
    local wrappedMessage = {}
    local currentLine = ""

    for i = 1, string.len(message) do
        local char = string.sub(message, i, i)
        if string.len(currentLine) >= visibleChars then
            table.insert(wrappedMessage, currentLine)
            currentLine = ""
        end
        currentLine = currentLine .. char
    end

    if currentLine ~= "" then
        table.insert(wrappedMessage, currentLine)
    end

    local isFirstLine = true
    for _, line in ipairs(wrappedMessage) do
        if isFirstLine then
            table.insert(chatMessages, {sender = sender, message = line, color = color, colorAlt = colorAlt})
            isFirstLine = false
        else
            table.insert(chatMessages, {sender = "", message = line, color = color, colorAlt = colorAlt})
        end
        if #chatMessages > maxMessages then
            table.remove(chatMessages, 1)
        end
    end

    currentMessageIndex = math.max(1, #chatMessages - maxVisibleLines + 1)
end

local function lightenColor(r, g, b, threshold)
    local function brighten(c)
        return c < threshold and threshold or c
    end

    return brighten(r), brighten(g), brighten(b)
end

local function renderChat()
    local startY = 128
    currentMessageIndex = math.min(math.max(1, currentMessageIndex), #chatMessages - maxVisibleLines + 1)
    if #chatMessages <= 0 then return end

    local startIdx = currentMessageIndex
    local endIdx = math.min(#chatMessages, startIdx + maxVisibleLines - 1)

    local minaColorThreshold = 191
    local minaAltColorThreshold = 63
    local color = 0
    local colorAlt = 0

    for i = startIdx, endIdx do
        local msg = chatMessages[i]
        if msg then
            if msg.sender ~= "" then
                local senderR, senderG, senderB = getColorComponents(msg.colorAlt or colorAlt)
                senderR, senderG, senderB = lightenColor(senderR, senderG, senderB, minaAltColorThreshold)
                GuiColorSetForNextWidget(gui, senderR / 255, senderG / 255, senderB / 255, 1)

                local senderText = string.format("%s: ", msg.sender)
                GuiText(gui, 64, startY, senderText)

                local senderWidth = calculateTextWidth(gui, string.format("%s: ", msg.sender))

                local textR, textG, textB = getColorComponents(msg.color or color)
                textR, textG, textB = lightenColor(textR, textG, textB, minaColorThreshold)
                GuiColorSetForNextWidget(gui, textR / 255, textG / 255, textB / 255, 1)

                GuiText(gui, 64 + senderWidth, startY, msg.message)
            else
                local textR, textG, textB = getColorComponents(msg.color or color)
                textR, textG, textB = lightenColor(textR, textG, textB, minaColorThreshold)
                GuiColorSetForNextWidget(gui, textR / 255, textG / 255, textB / 255, 1)
                GuiText(gui, 64, startY, msg.message)
            end
            startY = startY + lineHeight
        end
    end
end

local function renderTextInput()
    local startY = 128 + 150
    local wrappedMessage = {}
    local currentLine = ""

    if text == "" then
        GuiColorSetForNextWidget(gui, 0.5, 0.5, 0.5, 1)
        GuiText(gui, 64, startY, "ur text ll be here")
    else
        for i = 1, string.len(text) do
            local char = string.sub(text, i, i)
            if string.len(currentLine) >= visibleChars then
                table.insert(wrappedMessage, currentLine)
                currentLine = ""
            end
            currentLine = currentLine .. char
        end

        if currentLine ~= "" then
            table.insert(wrappedMessage, currentLine)
        end

        local maxLines = 8
        if #wrappedMessage > maxLines then
            wrappedMessage = {unpack(wrappedMessage, 1, maxLines)}
        end

        for _, line in ipairs(wrappedMessage) do
            GuiText(gui, 64, startY, line)
            startY = startY + lineHeight
        end
    end
end

local function disable_movement(controls)
    ComponentSetValue2(controls, "mButtonDownFire", false)
    ComponentSetValue2(controls, "mButtonDownFire2", false)
    ComponentSetValue2(controls, "mButtonDownLeft", false)
    ComponentSetValue2(controls, "mButtonDownDown", false)
    ComponentSetValue2(controls, "mButtonDownRight", false)
    ComponentSetValue2(controls, "mButtonDownUp", false)
    ComponentSetValue2(controls, "mButtonDownJump", false)
    ComponentSetValue2(controls, "mButtonDownFly", false)
    ComponentSetValue2(controls, "mButtonDownKick", false)
    ComponentSetValue2(controls, "mButtonDownEat", false)
end

function rpc.text(msg, color, colorAlt)
    if not ModSettingGet("quant.ew.notext") then
        GamePrint(ctx.rpc_player_data.name .. ": " .. msg)
        saveMessage(ctx.rpc_player_data.name, msg, color, colorAlt)
    end
end

local function starttext()
    ctx.is_texting = true
    local g = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "InventoryGuiComponent")
    if g ~= nil then
        EntitySetComponentIsEnabled(ctx.my_player.entity, g, false)
    end
    if not EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
        local controls = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "ControlsComponent")
        if controls ~= nil then
            ComponentSetValue2(controls, "enabled", false)
            disable_movement(controls)
        end
    end
end

local function stoptext()
    ctx.is_texting = false
    text = ""
    local g = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "InventoryGuiComponent")
    if g ~= nil then
        EntitySetComponentIsEnabled(ctx.my_player.entity, g, true)
    end
    if not EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
        local controls = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "ControlsComponent")
        if controls ~= nil then
            ComponentSetValue2(controls, "enabled", true)
        end
    end
end

function module.on_world_update()
    if InputIsMouseButtonJustDown(4) or InputIsKeyDown(82) then
        if #chatMessages > 0 then
            currentMessageIndex = math.max(1, currentMessageIndex - 1)
        end
    end
    
    if InputIsMouseButtonJustDown(5) or InputIsKeyDown(81) then
        if #chatMessages > 0 then
            currentMessageIndex = math.min(#chatMessages - maxVisibleLines + 1, currentMessageIndex + 1)
        end
    end
    if InputIsKeyJustDown(tonumber(ModSettingGet("quant.ew.text"))) then
        if ctx.is_texting == true then
            local non_white = false
            for i = 1, string.len(text) do
                if string.sub(text, i, i) ~= " " then
                    non_white = true
                    break
                end
            end
            if non_white then
                rpc.text(text, ctx.proxy_opt.mina_color_alt, ctx.proxy_opt.mina_color)
            end
            stoptext()
        else
            starttext()
        end
    end

    if ctx.is_texting == true and (InputIsKeyJustDown(tonumber(ModSettingGet("quant.ew.stoptext"))))
            or ctx.is_paused or ctx.is_wand_pickup then
        stoptext()
    end

    if ctx.is_texting == true then
        GuiStartFrame(gui)
        renderChat()
        renderTextInput()

        if string.len(text) > maxInputLength then
            text = string.sub(text, 1, maxInputLength)
        end

        local x, y = DEBUG_GetMouseWorld()
        x, y = world2gui(x, y)
        local new = GuiTextInput(gui, 421, x, y - 6, " ", 0, 256)
        if new ~= " " then
            if new == "" then
                text = string.sub(text, 1, -2)
            else
                text = text .. string.sub(new, 2, -1)
            end
        end
    end
end

return module
