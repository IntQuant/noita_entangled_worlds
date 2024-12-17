local module = {}
local gui = GuiCreate()
local rpc = net.new_rpc_namespace()

local text = ""

local unread_messages_counter = 0

rpc.opts_everywhere()
rpc.opts_reliable()

local chatMessages = {}
local maxVisibleLines = 128
local maxFileLines = 2048
local lineHeight = 10
local visibleLines = 15
local pixelWidth = 384
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

local function calculateTextWidth(msg)
    local width, _ = GuiGetTextDimensions(gui, msg)
    return width
end

local function getColorComponents(color)
    local b = math.floor(color / 2^16) % 2^8
    local g = math.floor(color / 2^8) % 2^8
    local r = color % 2^8
    return r, g, b
end

local function wrapText(gui, text, maxWidth, senderWidth)
    local wrappedLines = {}
    local currentLine = ""
    local isFirstLine = true
    local i = 1

    while i <= #text do
        local char = text:sub(i, i)
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

local function getFileLineCount(fileName)
    local lineCount = 0
    local file = io.open(fileName, "r")

    if file then
        for _ in file:lines() do
            lineCount = lineCount + 1
        end
        file:close()
    end

    return lineCount
end

local function trimFile(fileName)
    local file = io.open(fileName, "r")
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

    file = io.open(fileName, "w")
    for _, line in ipairs(lines) do
        file:write(line .. "\n")
    end
    file:close()
end

local function saveMessageToFile(sender, message, color, colorAlt)
    local fileName = "mods/quant.ew/files/system/text/chat_history.txt"
    local lineCount = getFileLineCount(fileName)

    if lineCount >= maxFileLines then
        trimFile(fileName)
    end

    local file = io.open(fileName, "a")
    local line
    if sender == "" then
        line = string.format("[%s,%s] : %s\n", color, colorAlt, message)
    else
        line = string.format("[%s,%s] %s: %s\n", color, colorAlt, sender, message)
    end
    file:write(line)
    file:close()
end

local function isFileEmpty(fileName)
    local file = io.open(fileName, "r")
    if not file then
        return true
    end
    
    local firstLine = file:read("*l")
    file:close()
    
    return firstLine == nil
end

local function copyPresetChatHistory()
    local presetFileName = "mods/quant.ew/files/system/text/chat_history_preset.txt"
    local chatHistoryFileName = "mods/quant.ew/files/system/text/chat_history.txt"

    local presetFile = io.open(presetFileName, "r")
    local chatHistoryFile = io.open(chatHistoryFileName, "a")

    if presetFile and chatHistoryFile then
        for line in presetFile:lines() do
            chatHistoryFile:write(line .. "\n")
        end
        presetFile:close()
        chatHistoryFile:close()
    end
end

local function loadChatHistory()
    local fileName = "mods/quant.ew/files/system/text/chat_history.txt"
    if isFileEmpty(fileName) then
        copyPresetChatHistory()
    end

    local file = io.open(fileName, "r")
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

local function saveMessage(sender, message, color, colorAlt)
    local senderWidth = sender ~= "" and GuiGetTextDimensions(gui, string.format("%s: ", sender)) or 0
    local wrappedMessage = wrapText(gui, message or "", pixelWidth, senderWidth)

    local maxLines = 8
    if #wrappedMessage > maxLines then
        wrappedMessage = {unpack(wrappedMessage, 1, maxLines)}
    end

    local isFirstLine = true
    for _, line in ipairs(wrappedMessage) do
        if isFirstLine then
            table.insert(chatMessages, {sender = sender, message = line, color = color, colorAlt = colorAlt})
            saveMessageToFile(sender, line, color, colorAlt)
            isFirstLine = false
        else
            table.insert(chatMessages, {sender = "", message = line, color = color, colorAlt = colorAlt})
            saveMessageToFile("", line, color, colorAlt)
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

    if #chatMessages <= 0 then return end

    local startIdx = currentMessageIndex
    local endIdx = math.min(#chatMessages, startIdx + visibleLines - 1)

    local minaColorThreshold = math.floor(ModSettingGet("quant.ew.textcolor") or 0)
    local minaAltColorThreshold = math.floor(ModSettingGet("quant.ew.textaltcolor") or 255)

    for i = startIdx, endIdx do
        local msg = chatMessages[i]
        if msg then
            local senderWidth = msg.sender ~= "" and GuiGetTextDimensions(gui, string.format("%s: ", msg.sender)) or 0
            local wrappedMessage = wrapText(gui, msg.message or "", pixelWidth, senderWidth)

            local senderRendered = false
            for _, line in ipairs(wrappedMessage) do
                if not senderRendered and msg.sender ~= "" then
                    local senderR, senderG, senderB = getColorComponents(msg.color)
                    senderR, senderG, senderB = lightenColor(senderR, senderG, senderB, minaColorThreshold)
                    GuiColorSetForNextWidget(gui, senderR / 255, senderG / 255, senderB / 255, 1)
                    GuiText(gui, 64, startY, string.format("%s: ", msg.sender))
                    senderRendered = true
                end

                local textR, textG, textB = getColorComponents(msg.colorAlt)
                textR, textG, textB = lightenColor(textR, textG, textB, minaAltColorThreshold)
                GuiColorSetForNextWidget(gui, textR / 255, textG / 255, textB / 255, 1)
                GuiText(gui, senderRendered and (64 + senderWidth) or 64, startY, line)
                startY = startY + lineHeight
            end
        end
    end
end

local function renderTextInput()
    local startY = 128 + 150

    if text == "" then
        GuiColorSetForNextWidget(gui, 0.5, 0.5, 0.5, 1)
        GuiText(gui, 64, startY, "Message *")
    else
        local wrappedMessage = wrapText(gui, text or "", pixelWidth, 0)

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

function rpc.text(msg, color, colorAlt)
    if not ModSettingGet("quant.ew.notext") then
        GamePrint(ctx.rpc_player_data.name .. ": " .. msg)
        saveMessage(ctx.rpc_player_data.name, msg, color, colorAlt)

        if ctx.rpc_player_data.name ~= ctx.my_player.name then
            unread_messages_counter = unread_messages_counter + 1
        end
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

local function create_chat_hint(hint) --maybe will make it for all hints in future if something else is added here for some reason
    local _, h = GuiGetScreenDimensions(gui)
    local _, th = GuiGetTextDimensions(gui, hint)
    GuiText(gui, 2, h-1-th, hint)
end

function module.on_world_update()
    if #chatMessages == 0 then
        loadChatHistory()
    end
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
                rpc.text(text, ctx.proxy_opt.mina_color, ctx.proxy_opt.mina_color_alt)
            end
            stoptext()
        else
            starttext()
        end
    end

    if ctx.is_texting == true and (InputIsKeyJustDown(tonumber(ModSettingGet("quant.ew.stoptext") or 76)))
            or ctx.is_paused or ctx.is_wand_pickup then
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

        if InputIsKeyJustDown(42) then -- backspace fix for russian letters
            local lastChar = string.sub(text, -1)
            local lastTwoChars = string.sub(text, -2)

            if lastChar:match("[\128-\191]") then
                text = string.sub(text, 1, -2)
                text = string.sub(text, 1, -1)
            else
                text = string.sub(text, 1, -2)
            end
            
            counter = 10
        end
        
        if InputIsKeyDown(42) then
            counter = counter + 1
            
            local lastChar = string.sub(text, -1)
            local lastTwoChars = string.sub(text, -2)
        
            if lastChar:match("[\128-\191]") then
                if counter == 3 then
                    text = string.sub(text, 1, -3)
                    counter = 0
                end
            else
                if counter == 3 then
                    text = string.sub(text, 1, -2)
                    counter = 0
                end
            end
            
            if counter == 30 then
                counter = 0
            end
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