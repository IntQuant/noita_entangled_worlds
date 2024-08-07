-- This module allows to handle getting damaged locally and redirects that damage to host.
-- Also handles shared health system in general.
-- Also recalculates percentage-based damage.

local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
local util = dofile_once("mods/quant.ew/files/src/util.lua")
local np = require("noitapatcher")

local rpc = net.new_rpc_namespace()

local module = {}

module.recent_damage = 0
module.recent_message = "unknown"
module.last_damage_message = "unknown"

ModLuaFileAppend("data/scripts/game_helpers.lua", "mods/quant.ew/files/src/system/damage/append/game_helpers.lua")
ModTextFileSetContent("data/entities/misc/effect_hearty.xml", ModTextFileGetContent("mods/quant.ew/files/src/system/damage/append/hearty_effect.xml"))

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
    local damage_model = EntityGetFirstComponentIncludingDisabled(my_player.entity, "DamageModelComponent")
    ComponentSetValue2(damage_model, "wait_for_kill_flag_on_death", true)
end

function module.on_world_update_client()
    
end

function module.on_world_update_host()
    
end

function module.on_new_player_seen(new_playerdata, player_count)
end

function module.on_client_spawned(peer_id, playerdata)
    if ctx.is_host then
        EntityAddComponent2(playerdata.entity, "LuaComponent", {script_damage_received = "mods/quant.ew/files/src/system/damage/cbs/send_damage_to_client.lua"})
    else
        EntityAddComponent2(playerdata.entity, "LuaComponent", {script_damage_about_to_be_received = "mods/quant.ew/files/cbs/immortal.lua"})
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

-- Provides health capability
ctx.cap.health = {
    health = module.health,
    max_health = module.max_health,
    set_health = module.set_health,
    set_max_health = module.set_max_health,
    inflict_damage = module.inflict_damage,
    do_game_over = function(message) do_game_over(message) rpc.trigger_game_over(message) end,
    on_poly_death = function() end,
}

rpc.opts_reliable()
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

return module
