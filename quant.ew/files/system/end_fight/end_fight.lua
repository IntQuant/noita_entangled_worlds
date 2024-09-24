local end_fight = {}
local first = true
local try_kill = -1
local wait_to_heal = false
local init = -1
local done = false

ModTextFileSetContent("data/entities/animals/boss_centipede/ending/gold_effect.xml", "<Entity/>")
ModTextFileSetContent("data/entities/animals/boss_centipede/ending/midas_sand.xml", "<Entity/>")
ModTextFileSetContent("data/entities/animals/boss_centipede/ending/midas_chunks.xml", "<Entity/>")

local function teleport_random()
    local r = Random(0, 7)
    local x, y = 6400, 15128
    if r == 0 then
        x, y = 6234, 14900
    elseif r == 1 then
        x, y = (2 * 6400) - 6234, 14900
    elseif r == 3 then
        x, y = 6184, 15170
    elseif r == 4 then
        x, y = (2 * 6400) - 6184, 15170
    elseif r == 5 then
        x, y = 6296, 15040
    elseif r == 6 then
        x, y = (2 * 6400) - 6296, 15040
    end
    EntitySetTransform(ctx.my_player.entity, x, y)
end

function end_fight.on_world_update()
    if GameHasFlagRun("ending_game_completed") and not done then
        if init == -1 then
            np.MagicNumbersSetValue("STREAMING_CHUNK_TARGET", 6)
            if EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
                EntityInflictDamage(ctx.my_player.entity, 100000000, "DAMAGE_CURSE", "", "None", 0, 0, GameGetWorldStateEntity())
                wait_to_heal = true
            else
                async(function()
                    wait(3)
                    EntityInflictDamage(ctx.my_player.entity, -100000000, "DAMAGE_HEALING", "", "None", 0, 0, GameGetWorldStateEntity())
                end)
            end
            GamePrintImportant("Fight for the spoils")
            first = false
            init = GameGetFrameNum() + 10
            teleport_random()
            LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/local_health/notplayer/safe_effect2.xml")
        elseif init < GameGetFrameNum() and GameGetFrameNum() % 10 == 0 then
            local exists = false
            for peer_id, playerdata in pairs(ctx.players) do
                if peer_id ~= ctx.my_id and not EntityHasTag(playerdata.entity, "ew_notplayer") then
                    exists = true
                    GenomeSetHerdId(playerdata.entity, "player_pvp")
                end
            end
            if wait_to_heal and not EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
                async(function()
                    wait(3)
                    EntityInflictDamage(ctx.my_player.entity, -100000000, "DAMAGE_HEALING", "", "None", 0, 0, GameGetWorldStateEntity())
                end)
                wait_to_heal = false
            end
            if not exists and not EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
                if try_kill <= GameGetFrameNum() then
                    local x, y = EntityGetTransform(ctx.my_player.entity)
                    EntityLoad("mods/quant.ew/files/system/end_fight/gold_effect.xml", x, y )
                    done = true
                elseif try_kill == -1 then
                    try_kill = GameGetFrameNum() + 180
                end
            else
                try_kill = -1
            end
            for _, player_data in pairs(ctx.players) do
                local entity = player_data.entity
                EntitySetComponentsWithTagEnabled(entity, "health_bar", false)
                EntitySetComponentsWithTagEnabled(entity, "health_bar_back", false)
                if EntityHasTag(entity, "ew_notplayer") then
                    for _, com in ipairs(EntityGetComponent(entity, "SpriteComponent") or {}) do
                        EntitySetComponentIsEnabled(entity, com, false)
                    end
                    for _, child in ipairs(EntityGetAllChildren(entity) or {}) do
                        EntityKill(child)
                    end
                end
            end
        end
    end
end

return end_fight