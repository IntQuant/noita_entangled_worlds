local end_fight = {}
local try_kill = -1
local init = -1
local done = false
local kill_walls = -1
local rpc = net.new_rpc_namespace()
dofile_once("data/scripts/status_effects/status_list.lua")

local status_effects = status_effects

ModTextFileSetContent("data/entities/animals/boss_centipede/ending/gold_effect.xml", "<Entity/>")
ModTextFileSetContent("data/entities/animals/boss_centipede/ending/midas_sand.xml", "<Entity/>")
ModTextFileSetContent("data/entities/animals/boss_centipede/ending/midas_chunks.xml", "<Entity/>")

local function remove_stuff(entity)
    for _, com in ipairs(EntityGetComponent(entity, "SpriteComponent") or {}) do
        EntitySetComponentIsEnabled(entity, com, false)
    end
    EntityRemoveTag(entity, "ew_peer")
    EntityRemoveTag(entity, "ew_client")
    EntityRemoveTag(entity, "mortal")
    EntityRemoveTag(entity, "homing_target")
    EntityRemoveTag(entity, "hittable")
    local collision = EntityGetFirstComponentIncludingDisabled(entity, "PlayerCollisionComponent")
    local suck = EntityGetFirstComponentIncludingDisabled(entity, "MaterialSuckerComponent")
    local gui = EntityGetFirstComponentIncludingDisabled(entity, "InventoryGuiComponent")
    local damage = EntityGetFirstComponentIncludingDisabled(entity, "DamageModelComponent")
    local genome = EntityGetFirstComponentIncludingDisabled(entity, "GenomeDataComponent")
    local status = EntityGetFirstComponentIncludingDisabled(entity, "StatusEffectDataComponent")
    if gui ~= nil then
        EntitySetComponentIsEnabled(entity, gui, false)
    end
    if suck ~= nil then
        EntitySetComponentIsEnabled(entity, suck, false)
    end
    if collision ~= nil then
        EntitySetComponentIsEnabled(entity, collision, false)
    end
    if damage ~= nil then
        EntitySetComponentIsEnabled(entity, damage, false)
    end
    if genome ~= nil then
        EntitySetComponentIsEnabled(entity, genome, false)
    end
    if status ~= nil then
        EntitySetComponentIsEnabled(entity, status, false)
    end
    for _, child in ipairs(EntityGetAllChildren(entity) or {}) do
        EntityKill(child)
    end
end

local function teleport_random()
    SetRandomSeed(5, 5)
    local my_num = Random(1,100)
    local keys = {}
    for key, _ in pairs(ctx.players) do
        table.insert(keys, key)
    end
    table.sort(keys)
    for i, key in ipairs(keys) do
        if key == ctx.my_id then
            my_num = my_num + i - 1
            break
        end
    end
    --local r = Random(0, 7)
    local x, y = 6400, 15128
    if my_num % 9 == 0 then
        x, y = 6234, 14900
    elseif my_num % 9 == 1 then
        x, y = (2 * 6398) - 6234, 14900
    elseif my_num % 9 == 2 then
        x, y = 6184, 15170
    elseif my_num % 9 == 3 then
        x, y = (2 * 6398) - 6184, 15170
    elseif my_num % 9 == 4 then
        x, y = 6296, 15040
    elseif my_num % 9 == 5 then
        x, y = (2 * 6398) - 6296, 15040
    elseif my_num % 9 == 6 then
        x, y = 6216, 15040
    elseif my_num % 9 == 7 then
        x, y = (2 * 6398) - 6216, 15040
    end
    EntitySetTransform(ctx.my_player.entity, x, y)
end

local winner

rpc.opts_everywhere()
function rpc.try_kill(x, y)
    EntityLoad("mods/quant.ew/files/system/end_fight/gold_effect.xml", x, y )
    done = true
    async(function()
        wait(300)
        if not ctx.run_ended then
            GameSetCameraFree(true)
            ctx.run_ended = true
            GameTriggerGameOver()
            for _, data in pairs(ctx.players) do
                EntityKill(data.entity)
            end
        end
    end)
    GamePrintImportant(ctx.rpc_player_data.name .. " wins")
    winner = ctx.rpc_peer_id
end

local function remove_fire(entity)
    local damage_model = EntityGetFirstComponentIncludingDisabled(entity, "DamageModelComponent")
    if damage_model ~= nil then
        ComponentSetValue2(damage_model, "mFireProbability", 0)
        ComponentSetValue2(damage_model, "mFireFramesLeft", 0)
    end
end

local function remove_status(entity, little)
    if EntityGetFirstComponent(entity, "StatusEffectDataComponent") == nil then
        return
    end
    if little then
        EntityRemoveStainStatusEffect(entity, status_effects[24].id)
        EntityRemoveIngestionStatusEffect(entity, status_effects[24].id)
    else
        for _, effect in pairs(status_effects) do
            if EntityGetIsAlive(entity) then
                EntityRemoveStainStatusEffect(entity, effect.id)
                EntityRemoveIngestionStatusEffect(entity, effect.id)
            end
        end
    end
    remove_fire(entity)
end

local function remove_game_effects()
    remove_status(ctx.my_player.entity, true)
    for _, child in ipairs(EntityGetAllChildren(ctx.my_player.entity) or {}) do
        local com = EntityGetFirstComponentIncludingDisabled(child, "GameEffectComponent")
        if com ~= nil and ComponentGetValue2(com, "effect") == "PROTECTION_ALL" then
            EntityKill(child)
        end
    end
end

local stop_fully = false

local first_death = {}

function end_fight.on_world_update()
    if stop_fully then
        return
    end
    if GameHasFlagRun("ending_game_completed") then
        if not done then
            if kill_walls == GameGetFrameNum() then
                for _, entity in pairs(EntityGetInRadius(6400, 15155, 100) or {}) do
                    if EntityGetFilename(entity) == "data/entities/animals/boss_centipede/ending/midas_walls.xml" then
                        EntityKill(entity)
                        break
                    end
                end
            end
            if init == -1 then
                if not GameHasFlagRun("ew_fight_started") then
                    GameAddFlagRun("ew_fight_started")
                else
                            local damage = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "DamageModelComponent")
        if damage ~= nil then
            ComponentSetValue2(damage, "hp", 2 ^ - 32)
        end
                    EntityInflictDamage(ctx.my_player.entity, 100000000, "DAMAGE_CURSE", "", "None", 0, 0, GameGetWorldStateEntity())
                end
                local _, y = EntityGetTransform(ctx.my_player.entity)
                np.MagicNumbersSetValue("STREAMING_CHUNK_TARGET", 6)
                np.MagicNumbersSetValue("GRID_FLEXIBLE_MAX_UPDATES", 1)
                if EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
                    remove_game_effects()
                            local damage = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "DamageModelComponent")
        if damage ~= nil then
            ComponentSetValue2(damage, "hp", 2 ^ - 32)
        end
                    EntityInflictDamage(ctx.my_player.entity, 100000000, "DAMAGE_CURSE", "", "None", 0, 0, GameGetWorldStateEntity())
                else
                    remove_game_effects()
                    EntityInflictDamage(ctx.my_player.entity, -100000000, "DAMAGE_HEALING", "", "None", 0, 0, GameGetWorldStateEntity())
                end
                GamePrintImportant("Fight for the spoils")
                init = GameGetFrameNum() + 10
                if y > 10414 then
                    teleport_random()
                end
                remove_fire(ctx.my_player.entity)
                kill_walls = GameGetFrameNum() + 180
            elseif init < GameGetFrameNum() and GameGetFrameNum() % 10 == 0 then
                local exists = false
                for peer_id, playerdata in pairs(ctx.players) do
                    if peer_id ~= ctx.my_id and not EntityHasTag(playerdata.entity, "ew_notplayer") then
                        exists = true
                        GenomeSetHerdId(playerdata.entity, "player_pvp")
                    end
                end
                if not exists and not EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
                    if try_kill <= GameGetFrameNum() and try_kill ~= -1 then
                        local x, y = EntityGetTransform(ctx.my_player.entity)
                        rpc.try_kill(x, y)
                        return
                    elseif try_kill == -1 then
                        try_kill = GameGetFrameNum() + 60
                    end
                else
                    try_kill = -1
                end
            end
        end
        if GameGetFrameNum() % 3 ~= 0 then
            return
        end
        for peer_id, player_data in pairs(ctx.players) do
            local entity = player_data.entity
            if not EntityGetIsAlive(entity) then
                goto continue
            end
            EntitySetComponentsWithTagEnabled(entity, "health_bar", false)
            EntitySetComponentsWithTagEnabled(entity, "health_bar_back", false)
            if EntityHasTag(entity, "ew_notplayer") then
                remove_stuff(entity)
                if first_death[peer_id] == nil and winner ~= ctx.my_id then
                    first_death[peer_id] = true
                    local x, y = EntityGetTransform(entity)
                    LoadRagdoll("mods/quant.ew/files/system/player/tmp/".. peer_id .."_ragdoll.txt", x, y)
                end
            end
            ::continue::
        end
    end
end

return end_fight