-- This module allows to handle getting damaged locally and redirects that damage to host.
-- Also handles shared health system in general.
-- Also recalculates percentage-based damage.

local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")
local util = dofile_once("mods/quant.ew/files/core/util.lua")
local inventory_helper = dofile_once("mods/quant.ew/files/core/inventory_helper.lua")
local np = require("noitapatcher")

local rpc = net.new_rpc_namespace()



local module = {}

function module.on_player_died(player_entity)
    -- This would be a good place to put on death logic
    -- BUT... player entity is already dead at this point, so it's a bit problematic to do stuff.
    -- Also inventory items seem to be borked.
end

local function player_died()
    -- Serialize inventory, we'll need to copy it over to notplayer.
    local item_data = inventory_helper.get_item_data(ctx.my_player)
    local _, max_hp = util.get_ent_health(ctx.my_player.entity)

    -- This may look like a hack, but it allows to use existing poly machinery to change player entity AND to store the original player for later,
    -- Which is, like, perfect.
    LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/local_health/notplayer/poly_effect.xml")

    GameAddFlagRun("ew_flag_notplayer_active")

    -- We kinda need to wait a frame for things to update.
    async(function ()
        wait(1)
        inventory_helper.set_item_data(item_data, ctx.my_player)
        util.set_ent_health(ctx.my_player.entity, {max_hp, max_hp})
    end)
end

local function do_game_over(message)
    net.proxy_notify_game_over()
    ctx.run_ended = true

    local damage_model = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "DamageModelComponent")
    ComponentSetValue2(damage_model, "wait_for_kill_flag_on_death", false)
    EntityInflictDamage(ctx.my_player.entity, 1000000, "DAMAGE_CURSE", message, "NONE", 0, 0, GameGetWorldStateEntity())
    GameTriggerGameOver()
    EntityKill(ctx.my_player.entity)
end

function module.on_local_player_spawn(my_player)
    util.set_ent_health(my_player.entity, {0.2, 4}) -- TODO remember to remove
    local damage_model = EntityGetFirstComponentIncludingDisabled(my_player.entity, "DamageModelComponent")
    ComponentSetValue2(damage_model, "wait_for_kill_flag_on_death", true)
    ctx.my_player.status = { is_alive = true }
end

function module.on_world_update()
    local notplayer_active = GameHasFlagRun("ew_flag_notplayer_active")
    if GameGetFrameNum() % 15 == 6 then
        local status = {
            is_alive = not notplayer_active
        }
        rpc.send_status(status)
    end
    
    local hp, max_hp, has_hp = util.get_ent_health(ctx.my_player.entity)
    if not ctx.my_player.currently_polymorphed and has_hp then
        if hp <= 0 then
            -- Restore the player back to small amount of hp.
            util.set_ent_health(ctx.my_player.entity, {1/25, max_hp})
            player_died()
        end
    end

    if notplayer_active then
        local controls = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "ControlsComponent")
        -- ComponentSetValue2(controls, "mButtonDownRight", true)
        -- ComponentSetValue2(controls, "mButtonFrameRight", GameGetFrameNum() + 1)
    end
end

function module.on_world_update_client()
    
end

-- Do not lose the game if there aren't any players alive from the start. (If alive players haven't connected yet)
local gameover_primed = false

function module.on_world_update_host()
    if GameGetFrameNum() % 60 == 15 then
        local any_player_alive = false
        for _, player_data in pairs(ctx.players) do
            local is_alive = player_data.status.is_alive
            if is_alive then
                gameover_primed = true
                any_player_alive = true
            end
        end
        if gameover_primed and not any_player_alive then
            rpc.trigger_game_over("No players are alive")
        end
    end
end

function module.on_new_player_seen(new_playerdata, player_count)
end

function module.on_client_spawned(peer_id, playerdata)
    playerdata.status = { is_alive = true }
    if ctx.is_host then
        EntityAddComponent2(playerdata.entity, "LuaComponent", {script_damage_received = "mods/quant.ew/files/system/damage/cbs/send_damage_to_client.lua"})
    else
        EntityAddComponent2(playerdata.entity, "LuaComponent", {script_damage_about_to_be_received = "mods/quant.ew/files/resource/cbs/immortal.lua"})
    end
end

function module.health()
end

function module.max_health()
end

function module.set_health(hp)
end

function module.set_max_health(hp)
end

function module.inflict_damage(dmg)
    local hp = module.health()
    module.set_health(math.min(math.max(hp-dmg, 0), module.max_health()))
end

local function end_poly_effect(ent)
    local children = EntityGetAllChildren(ent) or {}
    for _, child in pairs(children)do
        local game_effect_comp = EntityGetFirstComponentIncludingDisabled(child, "GameEffectComponent")
        if game_effect_comp then
            local effect = ComponentGetValue2(game_effect_comp, "effect")
            if effect == "POLYMORPH" then
                ComponentSetValue2(game_effect_comp, "frames", 1)
            end
        end
    end
end

-- Provides health capability
ctx.cap.health = {
    health = module.health,
    max_health = module.max_health,
    set_health = module.set_health,
    set_max_health = module.set_max_health,
    inflict_damage = module.inflict_damage,
    do_game_over = function(message) do_game_over(message) rpc.trigger_game_over(message) end,
    on_poly_death = function()
        local notplayer_active = GameHasFlagRun("ew_flag_notplayer_active")
        if notplayer_active then
            GameRemoveFlagRun("ew_flag_notplayer_active")
            end_poly_effect(ctx.my_player.entity)
        else
            end_poly_effect(ctx.my_player.entity)
            async(function ()
                wait(1)
                player_died()
            end)
        end
    end,
}

rpc.opts_reliable()
rpc.opts_everywhere()
function rpc.trigger_game_over(message)
    do_game_over(message)
end

rpc.opts_reliable()
function rpc.melee_damage_client(target_peer, damage, message)
    if ctx.my_player.peer_id == target_peer then
        EntityInflictDamage(ctx.my_player.entity, damage, "DAMAGE_MELEE", message, "NONE", 0, 0, 0)
    end
end
np.CrossCallAdd("ew_ds_client_damaged", rpc.melee_damage_client)

rpc.opts_everywhere()
function rpc.send_status(status)
    ctx.rpc_player_data.status = status
end

return module
