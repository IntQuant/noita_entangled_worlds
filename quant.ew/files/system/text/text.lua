local module = {}
local gui = GuiCreate()
local rpc = net.new_rpc_namespace()

local text = ""

local enabled = false

rpc.opts_everywhere()
rpc.opts_reliable()
function rpc.text(msg)
    local non_white = false
    for i = 1, #msg do
        if string.sub(msg, i, i) ~= " " then
            non_white = true
            break
        end
    end
    if not ModSettingGet("quant.ew.notext") and non_white then
        GamePrint(ctx.rpc_player_data.name .. ": " ..msg)
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

function module.on_world_update()
    if InputIsKeyJustDown(tonumber(ModSettingGet("quant.ew.text"))) then
        if enabled then
            rpc.text(string.sub(text, 1, -1))
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
        text = GuiTextInput(gui, 421, 64, 100, text, 512, 256)
    end
end

return module