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

function rpc.text(msg)
    if not ModSettingGet("quant.ew.notext") then
        GamePrint(ctx.rpc_player_data.name .. ": " .. msg)
        saveMessage(ctx.rpc_player_data.name, msg)
    end
end

local function starttext()
    enabled = true
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
    enabled = false
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

local first = true

function module.on_world_update()
    if InputIsKeyJustDown(tonumber(ModSettingGet("quant.ew.text"))) then
        if enabled then
            local non_white = false
            for i = 1, utf8len(text) do
                if utf8sub(text, i, i) ~= " " then
                    non_white = true
                    break
                end
            end
            if non_white then
                rpc.text(text)
            end
            stoptext()
            first = false
        else
            starttext()
        end
    end

    if enabled
            and (InputIsKeyJustDown(tonumber(ModSettingGet("quant.ew.stoptext")))
                or ctx.is_paused or ctx.is_wand_pickup) then
        stoptext()
        first = false
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
        if first then
            local w, h = GuiGetScreenDimensions(gui)
            local note = "text chat, hover over black box to type, enter to send"
            local tw, th = GuiGetTextDimensions(gui, note)
            GuiText(gui, w-2-tw, h-1-th, note)
        end
    end
end

return module