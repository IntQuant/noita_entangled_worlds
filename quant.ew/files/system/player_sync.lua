local rpc = net.new_rpc_namespace()

local module = {}

local last_money

local was_polied = false

local function undc(ent)
    local sprite = EntityGetFirstComponentIncludingDisabled(ent, "SpriteComponent")
    local name = ComponentGetValue2(sprite, "image_file")
    local new = string.sub(name, 0, -8) .. ".xml"
    ComponentSetValue2(sprite, "image_file", new)
    for _, child in ipairs(EntityGetAllChildren(ent) or {}) do
        if EntityGetName(child) == "notcursor" or EntityGetName(child) == "cursor" then
            sprite = EntityGetFirstComponentIncludingDisabled(child, "SpriteComponent")
            EntitySetComponentIsEnabled(child, sprite, true)
        end
    end
    EntityAddTag(ent, "ew_client")
    EntityAddTag(ent, "ew_peer")
end

function rpc.send_money_and_ingestion(money, delta, ingestion_size)
    local entity = ctx.rpc_player_data.entity
    local wallet = EntityGetFirstComponentIncludingDisabled(entity, "WalletComponent")
    if wallet ~= nil and money ~= nil then
        if ctx.proxy_opt.share_gold and last_money ~= nil then
            local my_wallet = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "WalletComponent")
            if my_wallet == nil then
                my_wallet = EntityAddComponent2(ctx.my_player.entity, "WalletComponent", { money = last_money })
            end
            local cm = ComponentGetValue2(my_wallet, "money")
            if cm ~= nil then
                if ctx.is_host then
                    ComponentSetValue2(my_wallet, "money", cm + delta)
                elseif ctx.rpc_peer_id == ctx.host_id then
                    local my_delta = 0
                    if cm ~= last_money then
                        my_delta = cm - last_money
                    end
                    last_money = money
                    ComponentSetValue2(my_wallet, "money", money + my_delta)
                end
            end
        end
        ComponentSetValue2(wallet, "money", money)
    end
    local ingestion = EntityGetFirstComponentIncludingDisabled(entity, "IngestionComponent")
    if ingestion ~= nil then
        ComponentSetValue2(ingestion, "ingestion_size", ingestion_size)
    end
end

local wait_on_send = 0

function rpc.request_items(peer_id)
    if ctx.my_id == peer_id and wait_on_send < GameGetFrameNum() then
        wait_on_send = GameGetFrameNum() + 240
        inventory_helper.has_inventory_changed(ctx.my_player)
        local inventory_state = player_fns.serialize_items(ctx.my_player)
        if inventory_state ~= nil then
            net.send_player_inventory(inventory_state)
        end
    end
end

local wait_on_requst = {}

local has_twwe_locally

function rpc.player_update(input_data, pos_data, phys_info, current_slot, team)
    local peer_id = ctx.rpc_peer_id

    if not player_fns.peer_has_player(peer_id) then
        player_fns.spawn_player_for(peer_id, pos_data.x, pos_data.y, team)
    end
    local player_data = player_fns.peer_get_player_data(peer_id)
    if player_data.dc then
        player_data.dc = false
        undc(player_data.entity)
        return
    end

    if
        team ~= nil
        and not GameHasFlagRun("ending_game_completed")
        and not EntityHasTag(player_data.entity, "polymorphed")
    then
        local my_team = tonumber(ModSettingGet("quant.ew.team")) or 0
        if my_team ~= -1 and team ~= -1 and (team == 0 or my_team == 0 or team ~= my_team) then
            GenomeSetHerdId(player_data.entity, "player_pvp")
        else
            GenomeSetHerdId(player_data.entity, "player")
        end
    end

    if input_data ~= nil then
        player_fns.deserialize_inputs(input_data, player_data)
    end
    if pos_data ~= nil then
        player_fns.deserialize_position(pos_data, phys_info, player_data)
    end
    if current_slot ~= nil then
        if
            not player_fns.set_current_slot(current_slot, player_data)
            and (wait_on_requst[player_data.peer_id] == nil or wait_on_requst[player_data.peer_id] < GameGetFrameNum())
        then
            print("slot empty, requesting items")
            wait_on_requst[player_data.peer_id] = GameGetFrameNum() + 300
            rpc.request_items(player_data.peer_id)
        end
    end
end

function rpc.check_gamemode(gamemode, seed, world_num, has_won)
    local mn = np.GetGameModeNr()
    local gm = np.GetGameModeName(mn)
    if gamemode == "save_slots_enabler" then
        gamemode = ""
    end
    if gamemode == "" then
        gamemode = "new run"
    end
    if gm == "save_slots_enabler" then
        gm = ""
    end
    if gm == "" then
        gm = "new run"
    end
    local not_fine = gamemode ~= gm
    local my_seed = StatsGetValue("world_seed")

    if not_fine then
        GamePrint("Player: " .. ctx.rpc_player_data.name .. ", is on a different gamemode number then you")
        GamePrint("their game mode: " .. gamemode)
        GamePrint("your game mode: " .. gm)
    end
    if my_seed ~= seed then
        GamePrint("Player: " .. ctx.rpc_player_data.name .. ", is on a different seed then you")
        GamePrint("their seed: " .. seed)
        GamePrint("your seed: " .. my_seed)
    end
    if world_num ~= ctx.proxy_opt.world_num then
        GamePrint("Player: " .. ctx.rpc_player_data.name .. ", is on a different world number then you")
        GamePrint("their num: " .. world_num)
        GamePrint("your num: " .. ctx.proxy_opt.world_num)
        GamePrint("world sync stops from this")
    end
    if has_won and not GameHasFlagRun("ending_game_completed") then
        GameAddFlagRun("ending_game_completed")
        GameAddFlagRun("ew_fight_started")
    end
end

local fps_last_frame = 0
local fps_last_update_time = 0

local function update_fps()
    local current_frame = GameGetFrameNum()
    local current_time = GameGetRealWorldTimeSinceStarted()
    local fps = (current_frame - fps_last_frame) / (current_time - fps_last_update_time)
    ctx.my_player.fps = math.min(60, math.floor(fps + 0.5))
    fps_last_frame = current_frame
    fps_last_update_time = current_time
end

function rpc.update_fps(fps)
    ewext.set_player_fps(ctx.rpc_peer_id, fps)
    ctx.rpc_player_data.fps = fps
end

local first = true

local find_later = false

local twwe_x

local twwe_y

function module.on_world_update()
    if first then
        local mn = np.GetGameModeNr()
        local gm = np.GetGameModeName(mn)
        net.send("join_notify", { ctx.my_player.name, gm })
        first = false
    end
    local input_data = player_fns.serialize_inputs(ctx.my_player)
    local pos_data, phys_info = player_fns.serialize_position(ctx.my_player)
    local current_slot = player_fns.get_current_slot(ctx.my_player)
    if input_data ~= nil or pos_data ~= nil then
        local my_team
        if ctx.proxy_opt.friendly_fire and GameGetFrameNum() % 10 == 7 then
            my_team = tonumber(ModSettingGet("quant.ew.team")) or 0
        end

        rpc.player_update(input_data, pos_data, phys_info, current_slot, my_team)
        if GameGetFrameNum() % 300 == 53 then
            local n = np.GetGameModeNr()
            rpc.check_gamemode(
                np.GetGameModeName(n),
                StatsGetValue("world_seed"),
                ctx.proxy_opt.world_num,
                GameHasFlagRun("ending_game_completed")
            )
        end
    end

    if GameGetFrameNum() % 32 == 7 then
        local mx, my = GameGetCameraPos()
        for peer_id, player in pairs(ctx.players) do
            local ent = player.entity
            local children = EntityGetAllChildren(ent) or {}
            if ctx.my_id ~= peer_id then
                for _, child in ipairs(children) do
                    if EntityGetName(child) == "cursor" then
                        if ModSettingGet("quant.ew.disable_cursors") then
                            local sprite = EntityGetFirstComponent(child, "SpriteComponent")
                            if sprite ~= nil and sprite ~= 0 then
                                EntitySetComponentIsEnabled(child, sprite, false)
                            end
                        elseif not player.dc then
                            EntitySetComponentIsEnabled(
                                child,
                                EntityGetFirstComponentIncludingDisabled(child, "SpriteComponent"),
                                true
                            )
                        end
                    end
                    if EntityGetName(child) == "notcursor" then
                        EntitySetComponentIsEnabled(
                            child,
                            EntityGetFirstComponentIncludingDisabled(child, "SpriteComponent"),
                            true
                        )
                    end
                end
            end
            local x, y = EntityGetTransform(ent)
            local notplayer = EntityHasTag(ent, "ew_notplayer")
                and not ctx.proxy_opt.perma_death
                and not ctx.proxy_opt.no_notplayer
            if notplayer and GameHasFlagRun("ending_game_completed") then
                goto continue
            end
            if x == nil or not EntityGetIsAlive(ent) or (not notplayer and EntityHasTag(ent, "polymorphed")) then
                goto continue
            end
            local dx, dy = x - mx, y - my
            local cape
            for _, child in ipairs(children) do
                if EntityGetName(child) == "cape" then
                    local cpe = EntityGetFirstComponentIncludingDisabled(child, "VerletPhysicsComponent")
                    local cx, cy = ComponentGetValue2(cpe, "m_position_previous")
                    local dcx, dcy = mx - cx, my - cy
                    if dcx * dcx + dcy * dcy > 350 * 350 then
                        EntityKill(child)
                    else
                        cape = child
                    end
                    break
                end
            end
            local light = EntityGetFirstComponentIncludingDisabled(ent, "LightComponent")
            if dx * dx + dy * dy > 350 * 350 then
                if cape ~= nil then
                    EntityKill(cape)
                end
                if light ~= nil then
                    EntitySetComponentIsEnabled(ent, light, false)
                end
            else
                if light ~= nil then
                    EntitySetComponentIsEnabled(ent, light, true)
                end
                if cape == nil then
                    local player_cape_sprite_file
                    if notplayer then
                        player_cape_sprite_file = "mods/quant.ew/files/system/local_health/notplayer/notplayer_cape.xml"
                    else
                        player_cape_sprite_file = "mods/quant.ew/files/system/player/tmp/" .. peer_id .. "_cape.xml"
                    end
                    local cape2 = EntityLoad(player_cape_sprite_file, x, y)
                    EntityAddChild(ent, cape2)
                end
            end
            ::continue::
        end
    end

    if GameGetFrameNum() % 37 == 9 then
        local wallet = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "WalletComponent")
        local ingestion = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "IngestionComponent")
        if was_polied ~= ctx.my_player.currently_polymorphed then
            if wallet ~= nil then
                ComponentSetValue2(wallet, "money", last_money)
            end
        end
        was_polied = ctx.my_player.currently_polymorphed
        if wallet ~= nil or ingestion ~= nil then
            local delta = 0
            if wallet == nil then
                wallet = EntityAddComponent2(ctx.my_player.entity, "WalletComponent", { money = last_money })
            end
            if wallet ~= nil then
                if last_money ~= nil then
                    delta = ComponentGetValue2(wallet, "money") - last_money
                end
                last_money = ComponentGetValue2(wallet, "money")
            end
            rpc.send_money_and_ingestion(
                last_money,
                delta,
                ingestion and ComponentGetValue2(ingestion, "ingestion_size")
            )
        end
    end

    if GameGetFrameNum() % 4 == 3 then
        local last = ctx.my_player.fps
        update_fps()
        if ctx.my_player.fps ~= last then
            ewext.set_player_fps(ctx.my_id, ctx.my_player.fps)
            rpc.update_fps(ctx.my_player.fps)
        end
    end

    if GameGetFrameNum() % 19 == 7 then
        if not EntityHasTag(ctx.my_player.entity, "polymorphed_player") then
            if find_later or (HasFlagPersistent("ew_twwe") and has_twwe_locally == nil) then
                find_later = false
                RemoveFlagPersistent("ew_twwe")
                for _, ent in ipairs(EntityGetAllChildren(ctx.my_player.entity)) do
                    local com = EntityGetFirstComponentIncludingDisabled(ent, "GameEffectComponent")
                    if
                        com ~= nil
                        and ComponentGetValue2(com, "effect") == "EDIT_WANDS_EVERYWHERE"
                        and not EntityHasTag(ent, "perk_entity")
                    then
                        RemoveFlagPersistent("ew_twwe")
                        EntityKill(ent)
                        break
                    end
                end
            end
            local x, y = EntityGetTransform(ctx.my_player.entity)
            if not ctx.my_player.twwe then
                local found = false
                for _, data in pairs(ctx.players) do
                    if data.twwe then
                        local tx, ty = EntityGetTransform(data.entity)
                        if tx ~= nil then
                            local dx, dy = tx - x, ty - y
                            if dx * dx + dy * dy < 20 * 20 then
                                found = true
                                break
                            end
                        end
                    end
                end
                if found then
                    twwe_x, twwe_y = x, y
                    if has_twwe_locally == nil then
                        AddFlagPersistent("ew_twwe")
                        has_twwe_locally = EntityLoad("mods/quant.ew/files/system/player/twwe.xml", twwe_x, twwe_y)
                        EntityAddChild(ctx.my_player.entity, has_twwe_locally)
                    end
                elseif has_twwe_locally ~= nil and (math.abs(twwe_x - x) >= 8 or math.abs(twwe_y - y) >= 8) then
                    RemoveFlagPersistent("ew_twwe")
                    EntityKill(has_twwe_locally)
                    has_twwe_locally = nil
                    twwe_x, twwe_y = nil, nil
                end
            elseif has_twwe_locally ~= nil and (math.abs(twwe_x - x) >= 8 or math.abs(twwe_y - y) >= 8) then
                RemoveFlagPersistent("ew_twwe")
                EntityKill(has_twwe_locally)
                has_twwe_locally = nil
                twwe_x, twwe_y = nil, nil
            end
        elseif has_twwe_locally ~= nil or HasFlagPersistent("ew_twwe") then
            find_later = true
            has_twwe_locally = nil
            twwe_x, twwe_y = nil, nil
        end
    end
end

return module
