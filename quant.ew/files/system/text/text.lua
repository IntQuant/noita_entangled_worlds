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

function module.on_world_update()
    if InputIsKeyJustDown(tonumber(ModSettingGet("quant.ew.text"))) then
        local controls = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "ControlsComponent")
        local g = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "InventoryGuiComponent")
        if enabled then
            rpc.text(string.sub(text, 1, -1))
            text = ""
            if gui ~= nil then
                EntitySetComponentIsEnabled(ctx.my_player.entity, g, true)
                ComponentSetValue2(controls, "enabled", true)
            end
        elseif gui ~= nil then
            EntitySetComponentIsEnabled(ctx.my_player.entity, g, false)
            ComponentSetValue2(controls, "enabled", false)
        end
        enabled = not enabled
    end
    if enabled then
        GuiStartFrame(gui)
        text = GuiTextInput(gui, 421, 64, 100, text, 512, 256)
    end
end

return module