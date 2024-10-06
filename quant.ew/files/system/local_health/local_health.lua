dofile_once("mods/quant.ew/files/system/player/player_cosmetics.lua")
local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")
local util = dofile_once("mods/quant.ew/files/core/util.lua")
local inventory_helper = dofile_once("mods/quant.ew/files/core/inventory_helper.lua")
local np = require("noitapatcher")
local perk_fns = dofile_once("mods/quant.ew/files/core/perk_fns.lua")
local nickname = dofile_once("mods/quant.ew/files/system/nickname.lua")

local rpc = net.new_rpc_namespace()

dofile_once("data/scripts/status_effects/status_list.lua")

local status_effects = status_effects

local module = {}

function module.on_player_died(player_entity)
    -- This would be a good place to put on death logic
    -- BUT... player entity is already dead at this point, so it's a bit problematic to do stuff.
    -- Also inventory items seem to be borked.
end

local function do_switch_effect(short)
    -- Make an effect
    if not EntityGetIsAlive(ctx.my_player.entity) then
        return
    end
    local x, y = EntityGetTransform(ctx.my_player.entity)
    rpc.switch_effect(x, y, short)
    if short then
        LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/local_health/notplayer/safe_effect2.xml")
    else
        LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/local_health/notplayer/safe_effect.xml")
    end
end

local function remove_inventory_tags()
    if not EntityGetIsAlive(ctx.my_player.entity) then
        return
    end
    local items = GameGetAllInventoryItems(ctx.my_player.entity)
    for _, item in ipairs(items) do
        EntityRemoveTag(item, "ew_client_item")
    end
end

local function remove_inventory()
    local children = EntityGetAllChildren(ctx.my_player.entity)
    for _, child in pairs(children) do
        if EntityGetName(child) == "inventory_quick" then
            local inv = EntityGetAllChildren(child)
            if inv ~= nil then
                for _, item in pairs(inv) do
                    EntityKill(item)
                end
            end
        end
    end
end

function rpc.add_nickname(id)
    nickname.add_label(ctx.players[id].entity, ctx.players[id].name, "mods/quant.ew/files/resource/font_pixel_runes.xml", 0.75, 0.75)
end

local function remove_healthbar_locally()
    EntitySetComponentsWithTagEnabled(ctx.my_player.entity, "health_bar", false)
    EntitySetComponentsWithTagEnabled(ctx.my_player.entity, "health_bar_back", false)
end


local function allow_notplayer_perk(perk_id)
    local ignored_perks = {
        GAMBLE = true,
        PERKS_LOTTERY = true,
        MEGA_BEAM_STONE = true,
        ALWAYS_CAST = true,
        EXTRA_SLOTS = true,
        EXTRA_PERK = true,
        EXTRA_SHOP_ITEM = true,
        FASTER_WANDS = true,
        EXTRA_MANA = true,
        RESPAWN = true,
        GENOME_MORE_HATRED = true,
        GENOME_MORE_LOVE = true,
        ESSENCE_LASER = true,
        ESSENCE_FIRE = true,
        ESSENCE_WATER = true,
        LUKKI_MINION = true,
        CONTACT_DAMAGE = true,
        FOOD_CLOCK = true, -- TODO, should carry over satiation buff
        TELEPORTITIS = true, -- TODO: teleports to 0,0
        TELEPORTITIS_DODGE = true,
    }
    return not ignored_perks[perk_id]
end

function rpc.change_cursor()
    for _, child in ipairs(EntityGetAllChildren(ctx.rpc_player_data.entity) or {}) do
        if (EntityGetName(child) == "cursor") then
            local sprite = EntityGetFirstComponentIncludingDisabled(child, "SpriteComponent")
            ComponentSetValue2(sprite, "image_file", "mods/quant.ew/files/system/player/tmp/" .. ctx.rpc_peer_id .. "_cursor.png")
            break
        end
    end
end

local function player_died()
    -- Serialize inventory, perks, and max_hp, we'll need to copy it over to notplayer.
    local item_data = inventory_helper.get_item_data(ctx.my_player)
    remove_inventory()
    local perk_data = perk_fns.get_my_perks()
    local _, max_hp = util.get_ent_health(ctx.my_player.entity)

    -- This may look like a hack, but it allows to use existing poly machinery to change player entity AND to store the original player for later,
    -- Which is, like, perfect.
    LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/local_health/notplayer/poly_effect.xml")

    -- We kinda need to wait a frame for things to update.
    async(function ()
        wait(1)
        GameSetCameraFree(true)
        GameAddFlagRun("ew_flag_notplayer_active")
        EntitySetName(ctx.my_player.entity, ctx.my_player.name.."?")
        do_switch_effect(false)
        inventory_helper.set_item_data(item_data, ctx.my_player)
        perk_fns.update_perks_for_entity(perk_data, ctx.my_player.entity, allow_notplayer_perk)
        util.set_ent_health(ctx.my_player.entity, {max_hp, max_hp})
        send_player_cosmetics(ctx.my_id)
        remove_inventory_tags()
        local iron = LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/local_health/notplayer/iron_stomach.xml")
        EntityAddTag(iron, "kill_on_revive")
        rpc.add_nickname(ctx.my_id)
        remove_healthbar_locally()
        for _, child in ipairs(EntityGetAllChildren(ctx.my_player.entity) or {}) do
            if EntityGetName(child) == "cursor" or EntityGetName(child) == "notcursor" then
                EntityKill(child)
            end
        end
        rpc.change_cursor()
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
    local damage_model = EntityGetFirstComponentIncludingDisabled(my_player.entity, "DamageModelComponent")
    ComponentSetValue2(damage_model, "wait_for_kill_flag_on_death", true)
    ctx.my_player.status = { is_alive = true }
end

function module.on_world_update()
    local notplayer_active = GameHasFlagRun("ew_flag_notplayer_active")
    local hp, max_hp = util.get_ent_health(ctx.my_player.entity)
    if GameGetFrameNum() % 15 == 6 then
        local status = {
            is_alive = not notplayer_active,
            hp = hp,
            max_hp = max_hp,
        }
        rpc.send_status(status)
    end

    local hp_new, max_hp_new, has_hp = util.get_ent_health(ctx.my_player.entity)
    if not ctx.my_player.currently_polymorphed and has_hp then
        if hp_new <= 0 then
            -- Restore the player back to small amount of hp.
            local new_hp = 3 * max_hp_new / 20
            local final_hp = math.max(new_hp, math.min(2/5, max_hp_new))
            util.set_ent_health(ctx.my_player.entity, {final_hp, max_hp_new})
            player_died()
        end
    end

    --if notplayer_active then
    --    local controls = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "ControlsComponent")
    --end
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
    local damage_model = EntityGetFirstComponentIncludingDisabled(playerdata.entity, "DamageModelComponent")
    ComponentSetValue2(damage_model, "wait_for_kill_flag_on_death", true)
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
            if effect == "POLYMORPH" or effect == "POLYMORPH_RANDOM" or effect == "POLYMORPH_UNSTABLE" or EntityHasTag(effect, "kill_on_revive") then
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
            if GameHasFlagRun("ending_game_completed") then
                return
            end
            local item_data = inventory_helper.get_item_data(ctx.my_player)
            remove_inventory()
            GameRemoveFlagRun("ew_flag_notplayer_active")
            end_poly_effect(ctx.my_player.entity)
            async(function ()
                wait(1)
                for _, child in pairs(EntityGetAllChildren(ctx.my_player.entity) or {}) do
                    if not EntityHasTag(child, "perk_entity") then
                        local com = EntityGetFirstComponentIncludingDisabled(child, "GameEffectComponent")
                        if com ~= nil or EntityHasTag(child, "projectile") then
                            EntityKill(child)
                        end
                    end
                end
                for _, effect in pairs(status_effects) do
                    if EntityGetIsAlive(ctx.my_player.entity) then
                        EntityRemoveStainStatusEffect(ctx.my_player.entity, effect.id)
                        EntityRemoveIngestionStatusEffect(ctx.my_player.entity, effect.id)
                    end
                end
                local damage_model = EntityGetFirstComponentIncludingDisabled(entity, "DamageModelComponent")
                if damage_model ~= nil then
                    ComponentSetValue2(damage_model, "mFireProbability", 0)
                    ComponentSetValue2(damage_model, "mFireFramesLeft", 0)
                end
                inventory_helper.set_item_data(item_data, ctx.my_player)
                remove_inventory_tags()
                local controls = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "ControlsComponent")
                if controls ~= nil then
                    ComponentSetValue2(controls, "enabled", true)
                end
                if GameHasFlagRun("ew_kill_player") then
                    GameRemoveFlagRun("ew_kill_player")
                    wait(100)
                    EntityInflictDamage(ctx.my_player.entity, 1000000, "DAMAGE_CURSE", "dont rejoin", "NONE", 0, 0, GameGetWorldStateEntity())
                else
                    do_switch_effect(true)
                end
            end)
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
    for _, player_data in pairs(ctx.players) do
        local entity = player_data.entity
        EntitySetComponentsWithTagEnabled(entity, "health_bar", false)
        EntitySetComponentsWithTagEnabled(entity, "health_bar_back", false)
        if EntityHasTag(entity, "ew_notplayer") then
            for _, com in ipairs(EntityGetComponent(entity, "SpriteComponent") or {}) do
                EntitySetComponentIsEnabled(entity, com, false)
            end
            local suck = EntityGetFirstComponentIncludingDisabled(entity, "MaterialSuckerComponent")
            local collision = EntityGetFirstComponentIncludingDisabled(entity, "PlayerCollisionComponent")
            EntitySetComponentIsEnabled(entity, suck, false)
            EntitySetComponentIsEnabled(entity, collision, false)
            for _, child in ipairs(EntityGetAllChildren(entity) or {}) do
                EntityKill(child)
            end
            for _, effect in pairs(status_effects) do
                if EntityGetIsAlive(entity) then
                    EntityRemoveStainStatusEffect(entity, effect.id)
                    EntityRemoveIngestionStatusEffect(entity, effect.id)
                end
            end
            local damage_model = EntityGetFirstComponentIncludingDisabled(entity, "DamageModelComponent")
            if damage_model ~= nil then
                ComponentSetValue2(damage_model, "mFireProbability", 0)
                ComponentSetValue2(damage_model, "mFireFramesLeft", 0)
            end
        end
    end
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
    local hp, _, has_hp = util.get_ent_health(ctx.rpc_player_data.entity)
    if hp > status.hp then
        EntityInflictDamage(ctx.rpc_player_data.entity, hp - status.hp, "DAMAGE_CURSE", "hp update", "NONE", 0, 0, GameGetWorldStateEntity())
    end
    util.set_ent_health(ctx.rpc_player_data.entity, {status.hp, status.max_hp})
end

rpc.opts_everywhere()
function rpc.switch_effect(x, y, to_normal_player)
    if to_normal_player then
        EntityLoad("mods/quant.ew/files/system/local_health/entities/magical_symbol_player.xml", x, y)
    else
        EntityLoad("data/entities/particles/image_emitters/magical_symbol_fast.xml", x, y)
    end
end

return module