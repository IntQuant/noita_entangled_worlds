local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local util = dofile_once("mods/quant.ew/files/core/util.lua")

if load_imgui == nil then
    return {}
end

imgui = load_imgui({version="1.20.0", mod="Entangled Worlds Debug Mode"})

local module = {}

function module.on_local_player_spawn(my_player)
    
    local player_entity = my_player.entity
    -- EntitySetTransform(player_entity, 1333, 770)

    -- util.load_ephemerial("mods/quant.ew/files/resource/entities/client.xml", 512*3+20, 512*3+10)
    -- EntityLoad("mods/quant.ew/files/resource/entities/client.xml", 512*3+20, 512*3+10)

    ctx.cap.health.set_max_health(1000)
    ctx.cap.health.set_health(1000)
    -- util.set_ent_health(player_entity, {1000, 1000})
    local wallet = EntityGetFirstComponentIncludingDisabled(player_entity, "WalletComponent")
    ComponentSetValue2(wallet, "money", 100000)
    -- GameSetCameraFree(true)

    -- dofile_once("data/scripts/perks/perk.lua")
    -- local x, y = EntityGetFirstHitboxCenter(player_entity)
    -- perk_spawn(x, y, "LASER_AIM", true)
    -- perk_spawn(x-50, y, "GLASS_CANNON", true)
    -- perk_spawn(x-25, y, "EDIT_WANDS_EVERYWHERE", true)
    -- EntityLoad("data/entities/items/pickup/heart.xml", x-75, y-20)

end

function module.on_world_update_host()
    local hp, max_hp = util.get_ent_health(ctx.my_player.entity)
    if hp < max_hp / 2 then
        -- util.set_ent_health(ctx.my_player.entity, {max_hp, max_hp})
    end
end

local function fw_button(label)
    return imgui.Button(label, imgui.GetWindowWidth() - 15, 20)
end

local function tp_button(label, x, y)
    if fw_button(label) then
        async(function()
            local player_entity = ctx.my_player.entity
            EntitySetTransform(player_entity, x, y)
            wait(5)
            EntitySetTransform(player_entity, x, y)
        end)
    end
end

local function show_game_effects()
    local entity = ctx.my_player.entity
    for _, ent in ipairs(EntityGetAllChildren(entity) or {}) do
        local com = EntityGetFirstComponent(ent, "GameEffectComponent")
        if com ~= nil then
            local name = ComponentGetValue2(com, "effect")
            imgui.Text(name)
        end
    end
end

function module.on_world_update_post()
    if imgui.Begin("EW Debug stuff") then
        if imgui.CollapsingHeader("General") then
            if fw_button("Set time to day") then
                local gamestate = GameGetWorldStateEntity()
                local gcom = EntityGetFirstComponentIncludingDisabled(gamestate, "WorldStateComponent")
                ComponentSetValue2(gcom, "time", 0)
            end
        end
        if imgui.CollapsingHeader("Player") then
            local dcom = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "DamageModelComponent")
            if dcom ~= nil and dcom ~= 0 then
                local enabled = ComponentGetIsEnabled(dcom)
                local ret = imgui.Checkbox("Vulnerable", enabled)
                if ret then
                    EntitySetComponentIsEnabled(ctx.my_player.entity, dcom, not enabled)
                end
            end
        end
        if imgui.CollapsingHeader("Teleports") then
            tp_button("Starting area", 0, -100)
            tp_button("Vault", 0, 8600-20)
            tp_button("Portal to lab", 350.332, 12852.998)
            tp_button("Kolmi room", 3400, 13040)
            tp_button("The Work", 6300, 15155)
            tp_button("Meat realm", 7328, 9263)
            tp_button("Kivi", 7427, -4960)
            tp_button("Null altar", 14000, 7500)
            tp_button("Orb 0", 765.510, -1075.003)
            tp_button("Orb 7", 4262.749, 887.997)
            tp_button("Tree", -1901.962, -1405.003)
        end
        if imgui.CollapsingHeader("Game effects") then
            show_game_effects()
        end
        -- imgui.Text("lalala")
        imgui.End()
    end
end

return module
