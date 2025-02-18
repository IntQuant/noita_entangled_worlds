local module = {}
local gui = GuiCreate()
local rpc = net.new_rpc_namespace()

local text = ""
local cursorPos = 0

local LPressed = false
local RPressed = false

local unread_messages_counter = 0

rpc.opts_everywhere()
rpc.opts_reliable()

local chatMessages = {}
local maxMessages = 128
local lineHeight = 10
local maxVisibleLines = 15
local visibleChars = 384 -- it's actually pixel width now
local currentMessageIndex = 1

local in_camera_ref

local function world2gui(x, y)
    in_camera_ref = in_camera_ref or false

    local gui_n = GuiCreate()
    GuiStartFrame(gui_n)
    local w, h = GuiGetScreenDimensions(gui_n)
    GuiDestroy(gui_n)

    local vres_scaling_factor = w
        / (MagicNumbersGetValue("VIRTUAL_RESOLUTION_X") + MagicNumbersGetValue("VIRTUAL_RESOLUTION_OFFSET_X"))
    local cam_x, cam_y = GameGetCameraPos()
    x, y = w / 2 + vres_scaling_factor * (x - cam_x), h / 2 + vres_scaling_factor * (y - cam_y)

    return x, y, vres_scaling_factor
end

local function calculateTextWidth(msg)
    local width, _ = GuiGetTextDimensions(gui, msg)
    return width
end

local function getColorComponents(color)
    local b = math.floor(color / 2 ^ 16) % 2 ^ 8
    local g = math.floor(color / 2 ^ 8) % 2 ^ 8
    local r = color % 2 ^ 8
    return r, g, b
end

local function wrapText(msg, maxWidth, senderWidth)
    local wrappedLines = {}
    local currentLine = ""
    local isFirstLine = true
    local i = 1

    while i <= #msg do
        local char = msg:sub(i, i)
        local newLine = currentLine .. char
        local lineWidth = GuiGetTextDimensions(gui, newLine)

        if isFirstLine then
            lineWidth = lineWidth + senderWidth
        end

        if lineWidth > maxWidth then
            local lastSpace = currentLine:match("^.*()%s")
            if lastSpace then
                table.insert(wrappedLines, currentLine:sub(1, lastSpace - 1))
                currentLine = currentLine:sub(lastSpace + 1) .. char
            else
                table.insert(wrappedLines, currentLine)
                currentLine = char
            end
            isFirstLine = false
        else
            currentLine = newLine
        end
        i = i + 1
    end

    if currentLine ~= "" then
        table.insert(wrappedLines, currentLine)
    end

    return wrappedLines
end

local function saveMessage(sender, message, color, colorAlt)
    local senderWidth = sender ~= "" and calculateTextWidth(string.format("%s: ", sender)) or 0
    local wrappedMessage = wrapText(message or "", visibleChars, senderWidth)

    local isFirstLine = true
    for _, line in ipairs(wrappedMessage) do
        if isFirstLine then
            table.insert(chatMessages, { sender = sender, message = line, color = color, colorAlt = colorAlt })
            isFirstLine = false
        else
            table.insert(chatMessages, { sender = "", message = line, color = color, colorAlt = colorAlt })
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
    unread_messages_counter = 0

    local startY = 128
    currentMessageIndex = math.min(math.max(1, currentMessageIndex), #chatMessages - maxVisibleLines + 1)
    if #chatMessages <= 0 then
        return
    end

    local startIdx = currentMessageIndex
    local endIdx = math.min(#chatMessages, startIdx + maxVisibleLines - 1)

    local minaColorThreshold = math.floor(ModSettingGet("quant.ew.textcolor") or 0)
    local minaAltColorThreshold = math.floor(ModSettingGet("quant.ew.textaltcolor") or 255)
    local color = 0
    local colorAlt = 0

    for i = startIdx, endIdx do
        local msg = chatMessages[i]
        if msg then
            local senderWidth = msg.sender ~= "" and calculateTextWidth(string.format("%s: ", msg.sender)) or 0
            local wrappedMessage = wrapText(msg.message or "", visibleChars, senderWidth)

            local senderRendered = false
            for _, line in ipairs(wrappedMessage) do
                if not senderRendered and msg.sender ~= "" then
                    local senderR, senderG, senderB = getColorComponents(msg.color or color)
                    senderR, senderG, senderB = lightenColor(senderR, senderG, senderB, minaColorThreshold)
                    GuiColorSetForNextWidget(gui, senderR / 255, senderG / 255, senderB / 255, 1)

                    local senderText = string.format("%s: ", msg.sender)
                    GuiText(gui, 64, startY, senderText)

                    local textR, textG, textB = getColorComponents(msg.colorAlt or colorAlt)
                    textR, textG, textB = lightenColor(textR, textG, textB, minaAltColorThreshold)
                    GuiColorSetForNextWidget(gui, textR / 255, textG / 255, textB / 255, 1)

                    GuiText(gui, 64 + senderWidth, startY, line)
                    senderRendered = true
                else
                    local textR, textG, textB = getColorComponents(msg.colorAlt or colorAlt)
                    textR, textG, textB = lightenColor(textR, textG, textB, minaAltColorThreshold)
                    GuiColorSetForNextWidget(gui, textR / 255, textG / 255, textB / 255, 1)

                    GuiText(gui, 64, startY, line)
                end
                startY = startY + lineHeight
            end
        end
    end
end

local function renderTextInput()
    local charCounter = 0
    local startY = 128 + 150

    if text == "" then
        GuiColorSetForNextWidget(gui, 0.5, 0.5, 0.5, 1)
        GuiText(gui, 64, startY, "Message *")
    else
        local wrappedMessage = wrapText(text or "", visibleChars, 0)

        local maxLines = 8
        if #wrappedMessage > maxLines then
            wrappedMessage = { unpack(wrappedMessage, 1, maxLines) }
        end

        for _, line in ipairs(wrappedMessage) do
            local lineCharCounter = 0
            for _ = 1, string.len(line) do
                charCounter = charCounter + 1
                lineCharCounter = lineCharCounter + 1
                if charCounter == cursorPos then
                    GuiText(gui, 63 + calculateTextWidth(string.sub(line, 1, lineCharCounter)), startY + 2, "_")
                end
            end

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

function rpc.text(msg, color, colorAlt, tx, ty)
    local x, y = GameGetCameraPos()
    local r = tonumber(ModSettingGet("quant.ew.text_range") or 0) or 0
    local dx = x - tx
    local dy = y - ty
    if not ModSettingGet("quant.ew.notext") and (r == 0 or dx * dx + dy * dy <= r * r) then
        GamePrint(ctx.rpc_player_data.name .. ": " .. msg)
        saveMessage(ctx.rpc_player_data.name, msg, color, colorAlt)

        if ctx.rpc_player_data.name ~= ctx.my_player.name then
            unread_messages_counter = unread_messages_counter + 1
        end
    end
end

local function starttext()
    cursorPos = 0

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

local function create_chat_hint(hint) --maybe will make it for all hints in future if something else is added here for some reason
    local _, h = GuiGetScreenDimensions(gui)
    local _, th = GuiGetTextDimensions(gui, hint)
    GuiText(gui, 2, h - 1 - th, hint)
end

function string.insert(str1, str2, pos)
    return str1:sub(1, pos) .. str2 .. str1:sub(pos + 1)
end

local counterL

local counterR

local counterB

function module.on_world_update()
    local gui_started = false
    if not ModSettingGet("quant.ew.nochathint") then
        if unread_messages_counter > 0 then --prevents hint from appearing all the time (can be annoying) and just appear when there is some unread message
            GuiStartFrame(gui)
            gui_started = true
            create_chat_hint("Use 'Enter' to open chat.(" .. unread_messages_counter .. " unread messages)")
        end
    end

    if InputIsKeyJustDown(tonumber(ModSettingGet("quant.ew.text") or 40)) then
        if ctx.is_texting == true then
            local non_white = false
            for i = 1, string.len(text) do
                if string.sub(text, i, i) ~= " " then
                    non_white = true
                    break
                end
            end
            if non_white then
                local x, y = GameGetCameraPos()
                rpc.text(text, ctx.proxy_opt.mina_color, ctx.proxy_opt.mina_color_alt, x, y)
            end
            stoptext()
        else
            starttext()
        end
    end

    if
        ctx.is_texting == true and (InputIsKeyJustDown(tonumber(ModSettingGet("quant.ew.stoptext") or 76)))
        or ctx.is_paused
        or ctx.is_wand_pickup
    then
        stoptext()
    end

    if ctx.is_texting == true then
        if not gui_started then
            GuiStartFrame(gui)
        end
        renderChat()
        renderTextInput()

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

        if InputIsKeyJustDown(42) and cursorPos - 1 >= 0 then --backspace
            cursorPos = cursorPos - 1
            text = string.sub(text, 1, cursorPos) .. string.sub(text, cursorPos + 2, -1)
            counterB = 10
        elseif InputIsKeyDown(42) and cursorPos - 1 >= 0 then
            counterB = counterB + 1
            if counterB == 3 then
                cursorPos = cursorPos - 1
                text = string.sub(text, 1, cursorPos) .. string.sub(text, cursorPos + 2, -1)
                counterB = 0
            end
            if counterB == 30 then --delay for deleting only 1 character
                counterB = 0
            end
        end

        --home, end, left and right arrow implementation
        if InputIsKeyDown(74) then --home
            cursorPos = 0
        end
        if InputIsKeyDown(77) then --end
            cursorPos = #text
        end

        if InputIsKeyJustUp(80) then
            LPressed = false
        end
        if InputIsKeyJustUp(79) then
            RPressed = false
        end

        if not InputIsKeyDown(224) then
            if InputIsKeyJustDown(80) and cursorPos - 1 >= 0 then --left arrow
                cursorPos = cursorPos - 1
                counterL = 10

                LPressed = true
                RPressed = false
            elseif not RPressed and InputIsKeyDown(80) and cursorPos - 1 >= 0 then
                counterL = counterL + 1
                if counterL == 3 then
                    cursorPos = cursorPos - 1
                    counterL = 0
                end
                if counterL == 30 then --delay for moving only 1 character
                    counterL = 0
                end
            end
            if InputIsKeyJustDown(79) and cursorPos + 1 < #text + 1 then --right arrow
                cursorPos = cursorPos + 1
                counterR = 10

                RPressed = true
                LPressed = false
            elseif not LPressed and InputIsKeyDown(79) and cursorPos + 1 < #text + 1 then
                counterR = counterR + 1
                if counterR == 3 then
                    cursorPos = cursorPos + 1
                    counterR = 0
                end
                if counterR == 30 then --delay for moving only 1 character
                    counterR = 0
                end
            end
        else
            local moveLength = (string.find(string.reverse(string.sub(text, 1, cursorPos)), " ") or cursorPos)
            if InputIsKeyJustDown(80) and cursorPos - moveLength >= 0 then --left arrow
                cursorPos = cursorPos - moveLength
                counterL = 10

                LPressed = true
                RPressed = false
            elseif not RPressed and InputIsKeyDown(80) and cursorPos - moveLength >= 0 then
                counterL = counterL + 1
                if counterL == 3 then
                    cursorPos = cursorPos - moveLength
                    counterL = 0
                end
                if counterL == 30 then --delay for moving only 1 character
                    counterL = 0
                end
            end
            moveLength = (string.find(string.sub(text, cursorPos + 2, -1), " ") or #text - cursorPos)
            if InputIsKeyJustDown(79) and cursorPos + moveLength < #text + 1 then --right arrow
                cursorPos = cursorPos + moveLength

                counterR = 10

                RPressed = true
                LPressed = false
            elseif not LPressed and InputIsKeyDown(79) and cursorPos + moveLength < #text + 1 then
                counterR = counterR + 1
                if counterR == 3 then
                    cursorPos = cursorPos + moveLength

                    counterR = 0
                end
                if counterR == 30 then --delay for moving only 1 character
                    counterR = 0
                end
            end
        end

        local x, y = DEBUG_GetMouseWorld()
        x, y = world2gui(x, y)
        local new = GuiTextInput(gui, 421, x, y - 6, " ", 0, 256)
        if new ~= " " then
            text = string.insert(text, string.sub(new, 2, -1), cursorPos)
            --text = text .. string.sub(new, 2, -1)
            cursorPos = cursorPos + #string.sub(new, 2, -1)
        end
    end
end

return module
