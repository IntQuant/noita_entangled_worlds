local module = {}
local gui = GuiCreate()
local rpc = net.new_rpc_namespace()

local text = ""
local cursorPos = 0

local LPressed = false
local RPressed = false

local unread_messages_counter = 0

local chatMessages = {}
local maxVisibleLines = 128
local maxFileLines = 2048
local lineHeight = 10
local visibleLines = 15
local pixelWidth = 384
local currentMessageIndex = 1
local maxLines = 8

local in_camera_ref

local appdata = os.getenv("APPDATA")
local chatHistoryFileName = appdata .. "/Noita Proxy/data/ew_chat.txt"

-- create path to chat history file with 2 cmd jumpscares
os.execute('mkdir "' .. appdata .. '\\Noita Proxy" 2>nul')
os.execute('mkdir "' .. appdata .. '\\Noita Proxy\\data" 2>nul')

local function world2gui(x, y)
    in_camera_ref = in_camera_ref or false

    local w, h = GuiGetScreenDimensions(gui)

    local vres_scaling_factor = w
        / (MagicNumbersGetValue("VIRTUAL_RESOLUTION_X") + MagicNumbersGetValue("VIRTUAL_RESOLUTION_OFFSET_X"))
    local cam_x, cam_y = GameGetCameraPos()
    x, y = w / 2 + vres_scaling_factor * (x - cam_x), h / 2 + vres_scaling_factor * (y - cam_y)

    return x, y, vres_scaling_factor
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

local function getFileLineCount(chatHistoryFileName)
    local lineCount = 0
    local file = io.open(chatHistoryFileName, "r")

    if file then
        for _ in file:lines() do
            lineCount = lineCount + 1
        end
        file:close()
    end

    return lineCount
end

local function trimFile(chatHistoryFileName)
    local file = io.open(chatHistoryFileName, "r")
    local lines = {}
    
    if file then
        for line in file:lines() do
            table.insert(lines, line)
        end
        file:close()
    end
    
    local removeCount = math.floor(#lines / 2)
    for i = 1, removeCount do
        table.remove(lines, 1)
    end

    file = io.open(chatHistoryFileName, "w")
    for _, line in ipairs(lines) do
        file:write(line .. "\n")
    end
    file:close()
end

local function saveMessageToFile(sender, message, color, colorAlt)
    local lineCount = getFileLineCount(chatHistoryFileName)

    if lineCount >= maxFileLines then
        trimFile(chatHistoryFileName)
    end

    local file = io.open(chatHistoryFileName, "a")
    local line
    if sender == "" then
        line = string.format("[%s,%s] : %s\n", color, colorAlt, message)
    else
        line = string.format("[%s,%s] %s: %s\n", color, colorAlt, sender, message)
    end
    file:write(line)
    file:close()
end

local function isFileEmpty(chatHistoryFileName)
    local file = io.open(chatHistoryFileName, "r")
    if not file then
        return true
    end
    
    local firstLine = file:read("*l")
    file:close()
    
    return firstLine == nil
end

local function copyPresetChatHistory()
    local presetFileName = "mods/quant.ew/files/system/text/chat_preset.txt"

    local presetFile = io.open(presetFileName, "r")
    chatHistoryFile = io.open(chatHistoryFileName, "a")

    if presetFile and chatHistoryFile then
        for line in presetFile:lines() do
            chatHistoryFile:write(line .. "\n")
        end
        presetFile:close()
        chatHistoryFile:close()
    end
end

local function loadChatHistory()
    if isFileEmpty(chatHistoryFileName) then
        copyPresetChatHistory()
    end

    local file = io.open(chatHistoryFileName, "r")
    local lines = {}
    local maxLinesToLoad = 128

    if file then
        local allLines = {}
        for line in file:lines() do
            table.insert(allLines, line)
        end
        file:close()

        local startIdx = math.max(1, #allLines - maxLinesToLoad + 1)
        for i = startIdx, #allLines do
            table.insert(lines, allLines[i])
        end
    end

    chatMessages = {}
    for _, line in ipairs(lines) do
        local color1, color2, sender, message = line:match("%[(%d+),(%d+)%] (%S+): (.*)")

        if not sender then
            sender = ""
            color1, color2, message = line:match("%[(%d+),(%d+)%] : (.*)")
        end

        if color1 and color2 and message then
            table.insert(chatMessages, {
                sender = sender,
                message = message,
                color = tonumber(color1),
                colorAlt = tonumber(color2),
            })
        end
    end

    currentMessageIndex = math.max(1, #chatMessages - visibleLines + 1)
end

if ModSettingGet("quant.ew.clearhistory") then
    chatMessages = {}
    os.remove(chatHistoryFileName)
    ModSettingSetNextValue("quant.ew.clearhistory", "false", false)
end

if not ModSettingGet("quant.ew.texthistory") and chatHistoryFile then
    os.remove(chatHistoryFileName)
end

local function saveMessage(sender, message, color, colorAlt)
    local senderWidth = sender ~= "" and GuiGetTextDimensions(gui, string.format("%s: ", sender)) or 0
    local wrappedMessage = wrapText(message or "", pixelWidth, senderWidth)

    if #wrappedMessage > maxLines then
        wrappedMessage = {unpack(wrappedMessage, 1, maxLines)}
    end

    local isFirstLine = true
    for _, line in ipairs(wrappedMessage) do
        if isFirstLine then
            table.insert(chatMessages, { sender = sender, message = line, color = color, colorAlt = colorAlt })
            if ModSettingGet("quant.ew.texthistory") then
                saveMessageToFile("", line, color, colorAlt)
            end
            isFirstLine = false
        else
            table.insert(chatMessages, { sender = "", message = line, color = color, colorAlt = colorAlt })
            if ModSettingGet("quant.ew.texthistory") then
                saveMessageToFile("", line, color, colorAlt)
            end
        end

        if #chatMessages > maxVisibleLines then
            table.remove(chatMessages, 1)
        end
    end

    currentMessageIndex = math.max(1, #chatMessages - visibleLines + 1)
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
    currentMessageIndex = math.min(math.max(1, currentMessageIndex), #chatMessages - visibleLines + 1)

    if #chatMessages <= 0 then
        return
    end

    local startIdx = currentMessageIndex
    local endIdx = math.min(#chatMessages, startIdx + visibleLines - 1)

    local minaColorThreshold = math.floor(ModSettingGet("quant.ew.textcolor") or 0)
    local minaAltColorThreshold = math.floor(ModSettingGet("quant.ew.textaltcolor") or 255)

    for i = startIdx, endIdx do
        local msg = chatMessages[i]
        if msg then
            local senderWidth = msg.sender ~= "" and GuiGetTextDimensions(gui, string.format("%s: ", msg.sender)) or 0
            local wrappedMessage = wrapText(msg.message or "", pixelWidth, senderWidth)

            local senderRendered = false
            for _, line in ipairs(wrappedMessage) do
                if not senderRendered and msg.sender ~= "" then
                    local senderR, senderG, senderB = getColorComponents(msg.color)
                    local lighten_senderR, lighten_senderG, lighten_senderB = lightenColor(senderR, senderG, senderB, minaColorThreshold)
                    GuiColorSetForNextWidget(gui, lighten_senderR / 255, lighten_senderG / 255, lighten_senderB / 255, 1)
                    GuiText(gui, 128, startY, string.format("%s: ", msg.sender))
                    senderRendered = true
                end

                local textR, textG, textB = getColorComponents(msg.colorAlt)
                local lighten_textR, lighten_textG, lighten_textB = lightenColor(textR, textG, textB, minaAltColorThreshold)
                GuiColorSetForNextWidget(gui, lighten_textR / 255, lighten_textG / 255, lighten_textB / 255, 1)
                GuiText(gui, senderRendered and (128 + senderWidth) or 128, startY, line)
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
        GuiText(gui, 128, startY, "Message *")
    else
        local wrappedMessage = wrapText(text or "", pixelWidth, 0)

        if #wrappedMessage > maxLines then
            wrappedMessage = { unpack(wrappedMessage, 1, maxLines) }
        end

        for _, line in ipairs(wrappedMessage) do
            local lineCharCounter = 0
            for _ = 1, string.len(line) do
                charCounter = charCounter + 1
                lineCharCounter = lineCharCounter + 1
                if charCounter == cursorPos then
                    GuiText(gui, 127 + GuiGetTextDimensions(gui, string.sub(line, 1, lineCharCounter)), startY + 2, "_")
                end
            end

            GuiText(gui, 128, startY, line)
            startY = startY + lineHeight
        end
    end
end

local function disable_movement(controls)
    local actions = {
        "mButtonDownFire",
        "mButtonDownFire2",
        "mButtonDownLeft",
        "mButtonDownDown",
        "mButtonDownRight",
        "mButtonDownUp",
        "mButtonDownJump",
        "mButtonDownFly",
        "mButtonDownKick",
        "mButtonDownEat"
    }

    for _, action in ipairs(actions) do
        ComponentSetValue2(controls, action, false)
    end
end

rpc.opts_everywhere()
rpc.opts_reliable()

function rpc.text(msg, color, colorAlt, tx, ty)
    local x, y = GameGetCameraPos()
    local r = tonumber(ModSettingGet("quant.ew.text_range") or 0) or 0
    local dx = x - tx
    local dy = y - ty
    if not ModSettingGet("quant.ew.notext") and (r == 0 or dx * dx + dy * dy <= r * r) then
        GamePrint(ctx.rpc_player_data.name .. ": " .. msg) -- it needs to be colored
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

local counterL = 0

local counterR = 0

local counterB = 0

local counterD = 0

local function utf8_next(str, pos)
    local c = str:byte(pos)
    if not c then
        return pos + 1
    end
    if c < 0x80 then
        return pos + 1
    elseif c < 0xE0 then
        return pos + 2
    elseif c < 0xF0 then
        return pos + 3
    elseif c < 0xF8 then
        return pos + 4
    else
        return pos + 1
    end
end

local function utf8_prev(str, pos)
    if pos <= 1 then
        return 1
    end
    pos = pos - 1
    while pos > 1 do
        local c = str:byte(pos)
        if c and (c < 0x80 or c >= 0xC0) then
            return pos
        end
        pos = pos - 1
    end
    return 1
end

function module.on_world_update()


    if #chatMessages == 0 and ModSettingGet("quant.ew.texthistory") then
        loadChatHistory()
    end
    local gui_started = false
    if not ModSettingGet("quant.ew.nochathint") then
        if unread_messages_counter > 0 then --prevents hint from appearing all the time (can be annoying) and just appear when there is some unread message
            GuiStartFrame(gui)
            gui_started = true
            if unread_messages_counter < 15 then
                create_chat_hint("Use 'Enter' to open chat.(" .. unread_messages_counter .. " unread messages)")
            end
            if unread_messages_counter >= 15 then
                create_chat_hint("READ CHAT NOW!!!(" .. unread_messages_counter .. " unread messages)")
            end
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
        ctx.is_texting == true and (InputIsKeyJustDown(tonumber(ModSettingGet("quant.ew.stoptext") or 277)))
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
            currentMessageIndex = math.max(1, currentMessageIndex - 1)
        end
        
        if InputIsMouseButtonJustDown(5) or InputIsKeyDown(81) then
            currentMessageIndex = math.min(currentMessageIndex + 1, #chatMessages - visibleLines + 1)
        end

        if InputIsKeyJustDown(42) and cursorPos - 1 >= 0 then -- backspace
            text = string.sub(text, 1, utf8_prev(text, cursorPos + 1) - 1) .. string.sub(text, cursorPos + 1)
            cursorPos = utf8_prev(text, cursorPos + 1) - 1
            if cursorPos < 0 then cursorPos = 0 end
            counterB = 10
        elseif InputIsKeyDown(42) and cursorPos - 1 >= 0 then
            counterB = counterB + 1
            if counterB == 3 then
                text = string.sub(text, 1, utf8_prev(text, cursorPos + 1) - 1) .. string.sub(text, cursorPos + 1)
                cursorPos = utf8_prev(text, cursorPos + 1) - 1
                if cursorPos < 0 then cursorPos = 0 end
                counterB = 0
            elseif counterB == 30 then -- delay for deleting only 1 character
                counterB = 0
            end
        end

        if InputIsKeyJustDown(76) and cursorPos < #text then -- delete
            text = string.sub(text, 1, cursorPos) .. string.sub(text, utf8_next(text, cursorPos + 1))
            counterD = 10
        elseif InputIsKeyDown(76) and cursorPos < #text then
            counterD = counterD + 1
            if counterD == 3 then
                text = string.sub(text, 1, cursorPos) .. string.sub(text, utf8_next(text, cursorPos + 1))
                counterD = 0
            elseif counterD == 30 then -- delay for deleting only 1 character
                counterD = 0
            end
        end

        -- home, end, left and right arrow implementation
        if InputIsKeyDown(74) then -- home
            cursorPos = 0
        end
        if InputIsKeyDown(77) then -- end
            cursorPos = #text
        end
        if InputIsKeyJustUp(80) then
            LPressed = false
        end
        if InputIsKeyJustUp(79) then
            RPressed = false
        end
        
        if not InputIsKeyDown(224) then
            if InputIsKeyJustDown(80) and cursorPos > 0 then -- left arrow
                cursorPos = utf8_prev(text, cursorPos + 1) - 1
                if cursorPos < 0 then cursorPos = 0 end
                counterL = 10
                LPressed = true
                RPressed = false
            elseif not RPressed and InputIsKeyDown(80) and cursorPos > 0 then
                counterL = counterL + 1
                if counterL == 3 then
                    cursorPos = utf8_prev(text, cursorPos + 1) - 1
                    if cursorPos < 0 then cursorPos = 0 end
                    counterL = 0
                elseif counterL == 30 then -- delay for moving only 1 character
                    counterL = 0
                end
            end
        
            if InputIsKeyJustDown(79) and cursorPos < #text then -- right arrow
                cursorPos = utf8_next(text, cursorPos + 1) - 1
                if cursorPos > #text then cursorPos = #text end
                counterR = 10
                RPressed = true
                LPressed = false
            elseif not LPressed and InputIsKeyDown(79) and cursorPos < #text then
                counterR = counterR + 1
                if counterR == 3 then
                    cursorPos = utf8_next(text, cursorPos + 1) - 1
                    if cursorPos > #text then cursorPos = #text end
                    counterR = 0
                elseif counterR == 30 then -- delay for moving only 1 character
                    counterR = 0
                end
            end
        else
            local moveLength = (string.find(string.reverse(string.sub(text, 1, cursorPos)), " ") or cursorPos)
            if InputIsKeyJustDown(42) and cursorPos - moveLength >= 0 then -- backspace with Ctrl
                text = string.sub(text, 1, cursorPos - moveLength) .. string.sub(text, cursorPos + 1)
                cursorPos = cursorPos - moveLength
                counterB = 10
            elseif InputIsKeyDown(42) and cursorPos - moveLength >= 0 then
                counterB = counterB + 1
                if counterB == 3 then
                    text = string.sub(text, 1, cursorPos - moveLength) .. string.sub(text, cursorPos + 1)
                    cursorPos = cursorPos - moveLength
                    counterB = 0
                end
                if counterB == 30 then -- delay for deleting only 1 character
                    counterB = 0
                end
            end
            if InputIsKeyJustDown(80) and cursorPos - moveLength >= 0 then -- left arrow with Ctrl
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
                if counterL == 30 then -- delay for moving only 1 character
                    counterL = 0
                end
            end
            local moveLength = (string.find(string.sub(text, cursorPos + 2, -1), " ") or #text - cursorPos)
            if InputIsKeyJustDown(76) and cursorPos < #text then -- delete with Ctrl
                text = string.sub(text, 1, cursorPos) .. string.sub(text, cursorPos + moveLength + 1)
                counterD = 10
            elseif InputIsKeyDown(76) and cursorPos < #text then
                counterD = counterD + 1
                if counterD == 3 then
                    text = string.sub(text, 1, cursorPos) .. string.sub(text, cursorPos + moveLength + 1)
                    counterD = 0
                end
                if counterD == 30 then -- delay for deleting only 1 character
                    counterD = 0
                end
            end
            if InputIsKeyJustDown(79) and cursorPos + moveLength < #text + 1 then -- right arrow with Ctrl
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
                if counterR == 30 then -- delay for moving only 1 character
                    counterR = 0
                end
            end
        end

        local x, y = DEBUG_GetMouseWorld()
        x, y = world2gui(x, y)
        local new = GuiTextInput(gui, 421, x, y - 6, " ", 0, 256)
        if new ~= " " then
            local updateText = string.insert(text, string.sub(new, 2, -1), cursorPos)

            local wrapped = wrapText(updateText, pixelWidth, 0)

            if #wrapped > maxLines then
                hamis = being_pet
            else
                text = updateText
                cursorPos = cursorPos + #string.sub(new, 2, -1)
            end
        end
    end
end

return module
