local end_fight = {}
local first = true
local try_kill = -1
local wait_to_heal = false
local init = -1

ModTextFileSetContent("data/entities/animals/boss_centipede/ending/gold_effect.xml", "<Entity/>")

function end_fight.on_world_update()
    if GameHasFlagRun("ending_game_completed") then
        if init == -1 then
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
        elseif init < GameGetFrameNum() then
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
                if try_kill == GameGetFrameNum() then
                    local x, y = EntityGetTransform(ctx.my_player.entity)
                    EntityLoad("mods/quant.ew/files/system/end_fight/gold_effect.xml", x, y )
                elseif try_kill == -1 then
                    try_kill = GameGetFrameNum() + 180
                end
            else
                try_kill = -1
            end
        end
    end
end

return end_fight