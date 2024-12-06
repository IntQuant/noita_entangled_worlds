local module = {}
local gui = GuiCreate()
local rpc = net.new_rpc_namespace()

local text = ""
local enabled = false

rpc.opts_everywhere()
rpc.opts_reliable()

local chatMessages = {}
local maxMessages = 100
local lineHeight = 10
local maxVisibleLines = 15
local maxInputLength = 1024
local visibleChars = 85

local function utf8len(s)
    local len = 0
    for _ in s:gmatch("[%z\1-\127\194-\244][\128-\191]*") do
        len = len + 1
    end
    return len
end

local function utf8sub(s, i, j)
    j = j or -1
    if i < 1 then i = utf8len(s) + i + 1 end
    if j < 1 then j = utf8len(s) + j + 1 end
    if i > j then return "" end
    local res = ""
    local k = 1
    for c in s:gmatch("[%z\1-\127\194-\244][\128-\191]*") do
        if k >= i and k <= j then res = res .. c end
        k = k + 1
    end
    return res
end

local function getOverflowText(message, startLimit)
    if utf8len(message) > startLimit then
        return utf8sub(message, startLimit + 1)
    else
        return ""
    end
end

local function saveMessage(sender, message)
    local wrappedMessage = {}
    local currentLine = ""

    for char in message:gmatch("[%z\1-\127\194-\244][\128-\191]*") do
        if utf8len(currentLine) >= visibleChars then
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
            table.insert(chatMessages, {sender = sender, message = line})
            isFirstLine = false
        else
            table.insert(chatMessages, {sender = "", message = line})
        end
        if #chatMessages > maxMessages then
            table.remove(chatMessages, 1)
        end
    end
end

local function renderChat()
    local startY = 128
    local startIdx = math.max(1, #chatMessages - maxVisibleLines + 1)

    for i = startIdx, #chatMessages do
        local msg = chatMessages[i]
        if msg.sender ~= "" then
            GuiText(gui, 64, startY, string.format("%s: %s", msg.sender, msg.message))
        else
            GuiText(gui, 64, startY, msg.message)
        end
        startY = startY + lineHeight
    end
end

function rpc.text(msg)
    if not ModSettingGet("quant.ew.notext") then
        local non_white = false
        for i = 1, utf8len(msg) do
            if utf8sub(msg, i, i) ~= " " then
                non_white = true
                break
            end
        end
        if non_white then
            GamePrint(ctx.rpc_player_data.name .. ": " .. msg)
            saveMessage(ctx.rpc_player_data.name, msg)
        end
    end
end

local function starttext()
    local controls = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "ControlsComponent")
    local g = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "InventoryGuiComponent")
    if g ~= nil then
        EntitySetComponentIsEnabled(ctx.my_player.entity, g, false)
    end
    if controls ~= nil then
        ComponentSetValue2(controls, "enabled", false)
    end
end

local function stoptext()
    text = ""
    local controls = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "ControlsComponent")
    local g = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "InventoryGuiComponent")
    if g ~= nil then
        EntitySetComponentIsEnabled(ctx.my_player.entity, g, true)
    end
    if controls ~= nil then
        ComponentSetValue2(controls, "enabled", true)
    end
end

function module.on_world_update()
    if InputIsKeyJustDown(tonumber(ModSettingGet("quant.ew.text"))) then
        if enabled then
            rpc.text(text)
            text = ""
            stoptext()
        else
            starttext()
        end
        enabled = not enabled
    end

    if enabled and (InputIsKeyJustDown(tonumber(ModSettingGet("quant.ew.stoptext"))) or ctx.is_paused or ctx.is_wand_pickup) then
        enabled = false
        stoptext()
    end

    if enabled then
        GuiStartFrame(gui)
        renderChat()

        if utf8len(text) > maxInputLength then
            text = utf8sub(text, 1, maxInputLength)
        end

        local overflowText = getOverflowText(text, visibleChars)
        GuiText(gui, 64, 115, overflowText)

        text = GuiTextInput(gui, 421, 64, 100, text, 512, 256)
    end
end

return module
