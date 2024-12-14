local rpc = net.new_rpc_namespace()

local best_times = rpc:create_var("best_times")

local karl = {}

local my_karl

local gui = GuiCreate()

function rpc.kill_karl()
    for _, entity in ipairs(EntityGetWithTag("racing_cart")) do
        local com = EntityGetFirstComponentIncludingDisabled(entity, "VariableStorageComponent", "ew_karl")
        if com ~= nil and ComponentGetValue2(com, "value_string") == ctx.rpc_peer_id then
            EntityKill(entity)
            break
        end
    end
end

function rpc.send_karl(x, y, vx, vy, t, jet, rgb1, rgb2)
    local players_karl
    for _, entity in ipairs(EntityGetWithTag("racing_cart")) do
        local com = EntityGetFirstComponentIncludingDisabled(entity, "VariableStorageComponent", "ew_karl")
        if com ~= nil and ComponentGetValue2(com, "value_string") == ctx.rpc_peer_id then
            if players_karl ~= nil then
                EntityKill(entity)
            else
                players_karl = entity
            end
        end
    end
    if players_karl == nil then
        players_karl = EntityLoad("data/entities/buildings/racing_cart.xml", x, y)
        EntitySetTransform(players_karl, x, y, t)
        EntityAddComponent2(players_karl, "VariableStorageComponent", {_tags = "ew_karl", value_string = ctx.rpc_peer_id})
        for _, com in ipairs(EntityGetComponent(players_karl, "LuaComponent")) do
            if ComponentGetValue2(com, "script_source_file") == "data/scripts/buildings/racing_cart_move.lua" then
                EntityRemoveComponent(players_karl, com)
            elseif ComponentGetValue2(com, "script_collision_trigger_hit") == "data/scripts/buildings/racing_cart_checkpoint.lua" then
                EntityRemoveComponent(players_karl, com)
            end
        end
        local particle = EntityGetComponentIncludingDisabled(players_karl, "ParticleEmitterComponent")
        local rgbc = rgb2 + 128 * 2^24
        ComponentSetValue2(particle[1], "color", rgbc)
        rgbc = rgb1 + 128 * 2^24
        ComponentSetValue2(particle[2], "color", rgbc)
    else
        EntitySetTransform(players_karl, x, y, t)
    end
    EntitySetComponentsWithTagEnabled(players_karl, "jetpack", jet)
    local vel = EntityGetFirstComponentIncludingDisabled(players_karl, "VelocityComponent")
    local m = ctx.rpc_player_data.fps / ctx.my_player.fps
    ComponentSetValue2(vel, "mVelocity", vx * m, vy * m)
end

local function draw_leaderboards_gui()
    GuiStartFrame(gui)
    GuiZSet(gui, 11)

    local _w, h = GuiGetScreenDimensions(gui)
    local text_x = 10
    local text_y = h / 5
    GuiText(gui, text_x, text_y - 10, "Lap times")
    for peer_id, peer_time in pairs(best_times.values) do
        GuiText(gui, text_x, text_y, string.format("%-16s %.02fs", player_fns.nickname_of_peer(peer_id), peer_time/60))
        text_y = text_y + 10
    end
end

function karl.on_world_update()
    if my_karl == nil or not EntityGetIsAlive(my_karl) then
        if my_karl ~= nil and not EntityGetIsAlive(my_karl) then
            rpc.kill_karl()
            my_karl = nil
        end
        if GameGetFrameNum() % 30 == 0 then
            for _, entity in ipairs(EntityGetWithTag("racing_cart")) do
                local com = EntityGetFirstComponentIncludingDisabled(entity, "VariableStorageComponent", "ew_karl")
                if com == nil then
                    my_karl = entity
                    local particle = EntityGetComponentIncludingDisabled(my_karl, "ParticleEmitterComponent")
                    local rgbc = ctx.proxy_opt.mina_color_alt + 128 * 2^24
                    ComponentSetValue2(particle[1], "color", rgbc)
                    rgbc = ctx.proxy_opt.mina_color + 128 * 2^24
                    ComponentSetValue2(particle[2], "color", rgbc)
                    break
                end
            end
        end
    else
        local x, y, t = EntityGetTransform(my_karl)
        local vel = EntityGetFirstComponentIncludingDisabled(my_karl, "VelocityComponent")
        local vx, vy = ComponentGetValue2(vel, "mVelocity")
        local jet = ComponentGetIsEnabled(EntityGetFirstComponentIncludingDisabled(my_karl, "SpriteParticleEmitterComponent"))
        rpc.send_karl(x, y, vx, vy, t, jet, ctx.proxy_opt.mina_color, ctx.proxy_opt.mina_color_alt)

        local stopwatch_best = EntityGetClosestWithTag(x, y, "stopwatch_best_lap")
        local com = EntityGetFirstComponentIncludingDisabled(stopwatch_best, "VariableStorageComponent")
        if com ~= nil then
            local best_time = ComponentGetValue2(com, "value_int")
            best_times.set(best_time)
        end
    end

    -- Center of the race track
    local center_x, center_y = 3200, 2312
    local x, y, w, h = GameGetCameraBounds()
    if x < center_x and center_x < x + w  and y < center_y and center_y < y+h then
        draw_leaderboards_gui()
    end

end

return karl