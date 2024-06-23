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

local function damage_received(damage, message, entity_id, add_healing_effect)
    local was_my_player = entity_id == nil or ctx.my_player.entity == entity_id
    if not was_my_player then
        return
    end
    
    module.recent_damage = module.recent_damage + damage
    if message ~= nil then
        module.recent_message = message
        module.last_damage_message = GameTextGetTranslatedOrNot(message) .. " from "..ctx.my_player.name
    end
    if ctx.is_host then
        module.inflict_damage(damage)
    end
    if add_healing_effect then
        rpc.healing_effect()
    end
end

np.CrossCallAdd("ew_ds_damaged", damage_received)

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
    util.ensure_component_present(my_player.entity, "LuaComponent", "ew_player_damage", {
        script_damage_received = "mods/quant.ew/files/src/system/damage/cbs/send_damage_to_host.lua"
    })
    if ctx.is_host then
    
    else
        -- ComponentSetValue2(damage_model, "damage_multipliers", "melee", 0) -- TODO
    end
    local damage_model = EntityGetFirstComponentIncludingDisabled(my_player.entity, "DamageModelComponent")
    ComponentSetValue2(damage_model, "wait_for_kill_flag_on_death", true)
    
end

function module.on_world_update_client()
    if module.recent_damage ~= 0 and GameGetFrameNum() % 15 == 2 then
        if not ctx.run_ended then
            rpc.deal_damage(module.recent_damage, module.recent_message)
            module.recent_damage = 0
            module.recent_message = "unknown"
        end
    end
end

local last_health = nil

local function do_health_diff(hp, max_hp)
    local current_hp = util.get_ent_health(ctx.my_player.entity)
    util.set_ent_health(ctx.my_player.entity, {hp, max_hp})
    if last_health ~= nil then
        local diff = last_health - current_hp
        if diff ~= 0 then
            damage_received(diff, nil)
        end
    end
    last_health = util.get_ent_health(ctx.my_player.entity)
end

function module.on_world_update_host()
    if GameGetFrameNum() % 4 == 3 then
        local hp, max_hp = module.health(), module.max_health()
        if not ctx.my_player.currently_polymorphed then
            do_health_diff(hp, max_hp)
        end
        rpc.update_shared_health(hp, max_hp)
        if hp <= 0 and not ctx.run_ended then
            local message = module.last_damage_message
            do_game_over(message)
            rpc.trigger_game_over(message)
        end
    end
end

function module.on_new_player_seen(new_playerdata)
    module.set_max_health(module.max_health()+4)
    module.set_health(module.health()+4)
end

function module.health()
    return tonumber(GlobalsGetValue("ew_shared_hp", "4"))
end

function module.max_health()
    return tonumber(GlobalsGetValue("ew_shared_max_hp", "4"))
end

function module.set_health(hp)
    GlobalsSetValue("ew_shared_hp", tostring(hp))
end

function module.set_max_health(hp)
    GlobalsSetValue("ew_shared_max_hp", tostring(hp))
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
    do_game_over = function(message) do_game_over(message) rpc.trigger_game_over(message) end
}

rpc.opts_reliable()
function rpc.deal_damage(damage, message)
    local message = GameTextGetTranslatedOrNot(message) .. " ("..ctx.rpc_player_data.name..")"
    module.last_damage_message = message
    if ctx.is_host then
        local host_entity_id = ctx.my_player.entity
        local protection_component_id = GameGetGameEffect(host_entity_id, "PROTECTION_ALL")
        if protection_component_id ~= 0 then
            EntitySetComponentIsEnabled(host_entity_id, protection_component_id, false)
        end
        
        module.inflict_damage(damage)
        if protection_component_id ~= 0 then
            EntitySetComponentIsEnabled(host_entity_id, protection_component_id, true)
        end
    end
    GamePrint("Got ".. (damage*25) .." damage: "..message)
end

function rpc.update_shared_health(hp, max_hp)
    if not ctx.my_player.currently_polymorphed then
        do_health_diff(hp, max_hp)
    end
end

rpc.opts_reliable()
function rpc.trigger_game_over(message)
    do_game_over(message)
end

function rpc.healing_effect()
    local entity_id = ctx.rpc_player_data.entity
    local x, y = EntityGetTransform( entity_id )
	local entity_fx = EntityLoad( "data/entities/particles/heal_effect.xml", x, y )
	EntityAddChild( entity_id, entity_fx )
end

return module
