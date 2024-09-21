local end_fight = {}
local first = true
local try_kill = -1
local wait_to_heal = false

ModLuaFileAppend("data/entities/animals/boss_centipede/ending/sampo_start_ending_sequence.lua", "mods/quant.ew/files/system/end_fight/ending_sequence_append.lua")

function end_fight.on_world_update()
    if GameHasFlagRun("ending_game_completed") then
        local exists = false
        for peer_id, playerdata in pairs(ctx.players) do
            if peer_id ~= ctx.my_id and not EntityHasTag(playerdata.entity, "ew_notplayer") then
                exists = true
                GenomeSetHerdId(playerdata.entity, "player_pvp")
            end
        end
        if wait_to_heal and not EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
            async(function()
                wait(1)
                EntityInflictDamage(ctx.my_player.entity, 100000000, "DAMAGE_HEALING", "", "None", 0, 0, GameGetWorldStateEntity())
            end)
            wait_to_heal = false
        end
        if first then
            if EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
                wait_to_heal = true
            else
                async(function()
                    wait(1)
                    EntityInflictDamage(ctx.my_player.entity, 100000000, "DAMAGE_HEALING", "", "None", 0, 0, GameGetWorldStateEntity())
                end)
            end
            GamePrintImportant("Fight for the spoils")
            first = false
        elseif not exists then
            if try_kill == GameGetFrameNum() then
                local x, y = EntityGetTransform(ctx.my_player.entity)
                EntityLoad("data/entities/animals/boss_centipede/ending/gold_effect.xml", x, y )
            elseif try_kill == -1 then
                try_kill = GameGetFrameNum() + 180
            end
        else
            try_kill = -1
        end
    end
end

return end_fight