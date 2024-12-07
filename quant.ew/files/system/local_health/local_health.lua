local polymorph = dofile_once("mods/quant.ew/files/system/polymorph/polymorph.lua")
local base64 = dofile_once("mods/quant.ew/files/resource/base64.lua")
local perk_fns = dofile_once("mods/quant.ew/files/core/perk_fns.lua")
local nickname = dofile_once("mods/quant.ew/files/system/nickname.lua")
local spectate = dofile_once("mods/quant.ew/files/system/spectate/spectate.lua")

local rpc = net.new_rpc_namespace()

dofile_once("data/scripts/status_effects/status_list.lua")

local status_effects = status_effects

local module = {}

local last_damage_info = {0, "unknown", 1}

util.add_cross_call("ew_damage_message", function(message, entity_thats_responsible)
    last_damage_info = {GameGetFrameNum(), message, entity_thats_responsible}
end)

function module.on_player_died(player_entity)
    -- This would be a good place to put on death logic
    -- BUT... player entity is already dead at this point, so it's a bit problematic to do stuff.
    -- Also inventory items seem to be borked.
end

local function get_gold()
    local wallet = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "WalletComponent")
    if wallet ~= nil then
        return ComponentGetValue2(wallet, "money")
    end
end

local function set_gold(gold)
    if gold ~= nil then
        local wallet = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "WalletComponent")
        if wallet ~= nil then
            ComponentSetValue2(wallet, "money", gold)
        end
    end
end

rpc.opts_everywhere()
function rpc.remove_homing(clear_area)
    local x, y
    if ctx.rpc_peer_id == ctx.my_id then
        x, y = EntityGetTransform(ctx.rpc_player_data.entity)
    else
        x, y = ctx.rpc_player_data.pos_x, ctx.rpc_player_data.pos_y
    end
    for _, proj in pairs(EntityGetInRadiusWithTag(x, y, 512, "player_projectile")) do
        local homing = EntityGetFirstComponentIncludingDisabled(proj, "HomingComponent")
        if homing ~= nil and ComponentGetValue2(homing, "target_tag") ~= "ew_peer" then
            EntitySetComponentIsEnabled(proj, homing, false)
        end
    end
    if clear_area then
        LoadPixelScene("mods/quant.ew/files/system/local_health/revive.png", "", x - 6, y - 13, "", true, true)
    end
end

local function remove_stuff()
    for _, child in ipairs(EntityGetAllChildren(ctx.my_player.entity) or {}) do
        if not EntityHasTag(child, "perk_entity") then
            local com = EntityGetFirstComponentIncludingDisabled(child, "GameEffectComponent")
            local comt = EntityGetFirstComponentIncludingDisabled(child, "LifetimeComponent")
            if (com ~= nil and ComponentGetValue2(com, "frames") ~= -1 and ComponentGetValue2(com, "frames") < 60 * 60 * 60)
                    or (comt ~= nil and ComponentGetValue2(comt, "lifetime") ~= -1 and ComponentGetValue2(comt, "lifetime") < 60 * 60 * 60)
                    or EntityHasTag(child, "projectile") then
                EntityKill(child)
            end
        end
    end
    if EntityGetFirstComponent(ctx.my_player.entity, "StatusEffectDataComponent") ~= nil then
        for _, effect in pairs(status_effects) do
            if EntityGetIsAlive(ctx.my_player.entity) then
                EntityRemoveStainStatusEffect(ctx.my_player.entity, effect.id)
                EntityRemoveIngestionStatusEffect(ctx.my_player.entity, effect.id)
            end
        end
    end
    local damage_model = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "DamageModelComponent")
    if damage_model ~= nil then
        ComponentSetValue2(damage_model, "mFireProbability", 0)
        ComponentSetValue2(damage_model, "mFireFramesLeft", 0)
        ComponentSetValue2(damage_model, "air_in_lungs", ComponentGetValue2(damage_model, "air_in_lungs_max"))
    end
end

local function set_camera_free(enable, entity, dont)
    local cam = EntityGetFirstComponentIncludingDisabled(entity, "PlatformShooterPlayerComponent")
    if cam ~= nil then
        ComponentSetValue2(cam, "center_camera_on_this_entity", not enable)
        ComponentSetValue2(cam, "move_camera_with_aim", not enable)
    elseif not dont then
        GameSetCameraFree(true)
    end
end

local function end_poly_effect(ent)
    local serialized
    for _, child in ipairs(EntityGetAllChildren(ent) or {}) do
        local game_effect_comp = EntityGetFirstComponentIncludingDisabled(child, "GameEffectComponent")
        if game_effect_comp then
            local effect = ComponentGetValue2(game_effect_comp, "effect")
            if effect == "POLYMORPH" or effect == "POLYMORPH_RANDOM"
                    or effect == "POLYMORPH_UNSTABLE" or effect == "POLYMORPH_CESSATION" then
                serialized = ComponentGetValue2(game_effect_comp, "mSerializedData")
                if serialized ~= nil then
                    break
                end
            end
        end
    end
    if serialized == nil then
        return
    end
    local x, y = EntityGetTransform(ent)
    local new_ent = util.deserialize_entity(base64.decode(serialized), x, y)
    GameSetCameraFree(true)
    np.SetPlayerEntity(new_ent)
    async(function()
        wait(1)
        GameSetCameraFree(false)
    end)
    EntityKill(ent)
    return new_ent
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
    for _, item in ipairs(items or {}) do
        EntityRemoveTag(item, "ew_client_item")
    end
end

local function remove_inventory()
    local children = EntityGetAllChildren(ctx.my_player.entity)
    for _, child in pairs(children or {}) do
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

rpc.opts_everywhere()
function rpc.add_nickname_change_cursor()
    nickname.add_label(ctx.rpc_player_data.entity, ctx.rpc_player_data.name, "mods/quant.ew/files/resource/font_pixel_runes.xml", 0.75, 0.75)

    if ctx.my_id ~= ctx.rpc_peer_id then
        LoadGameEffectEntityTo(ctx.rpc_player_data.entity, "mods/quant.ew/files/system/spectate/no_tinker.xml")
        for _, child in ipairs(EntityGetAllChildren(ctx.rpc_player_data.entity) or {}) do
            if (EntityGetName(child) == "cursor") then
                local sprite = EntityGetFirstComponentIncludingDisabled(child, "SpriteComponent")
                ComponentSetValue2(sprite, "image_file", "mods/quant.ew/files/system/player/tmp/" .. ctx.rpc_peer_id .. "_cursor.png")
                break
            end
        end
    end
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
        FASTER_WANDS = true,
        EXTRA_MANA = true,
        RESPAWN = true,
        ESSENCE_LASER = true,
        ESSENCE_FIRE = true,
        ESSENCE_WATER = true,
        LUKKI_MINION = true,
        CONTACT_DAMAGE = true,
        HEARTS_MORE_EXTRA_HP = true,
        REVENGE_RATS = true,
        FOOD_CLOCK = true, -- TODO, should carry over satiation buff
    }
    return not ignored_perks[perk_id]
end

local function reduce_hp()
    local p = 100 - ctx.proxy_opt.health_lost_on_revive
    if p ~= 100 then
        if ctx.proxy_opt.global_hp_loss then
            rpc.loss_hp()
        end
        local hp, max_hp = util.get_ent_health(ctx.my_player.entity)
        util.set_ent_health(ctx.my_player.entity, {(hp * p) / 100, (max_hp * p) / 100})
    end
end

rpc.opts_everywhere()
rpc.opts_reliable()
function rpc.show_death_message(untranslated_message, source_player)
    local message = "unknown"
    if untranslated_message ~= nil then
        message = GameTextGetTranslatedOrNot(untranslated_message)
    end
    if source_player ~= nil then
        message = message .. " from "..source_player
    end

    local dead_nickname = ctx.rpc_player_data.name
    local full_msg = dead_nickname .. " died: " .. message
    GamePrint(full_msg)
end

local function show_death_message()
    local current_frame = GameGetFrameNum()
    -- Check if message is recent enough
    if current_frame - last_damage_info[1] < 60 then
        local message = last_damage_info[2]
        local source
        local source_ent = last_damage_info[3]
        local maybe_player = player_fns.get_player_data_by_local_entity_id(source_ent)
        if maybe_player ~= nil then
            source = maybe_player.name
        end
        rpc.show_death_message(message, source)
    else
        rpc.show_death_message(nil, nil)
    end
end

local function reset_cast_state_if_has_any_other_item(player_data)
    local inventory2Comp = EntityGetFirstComponentIncludingDisabled(player_data.entity, "Inventory2Component")
    if inventory2Comp == nil then
        return
    end
    local mActiveItem = ComponentGetValue2(inventory2Comp, "mActiveItem")

    for k, item in ipairs(inventory_helper.get_inventory_items(player_data, "inventory_quick") or {}) do
        if item ~= mActiveItem then
            np.SetActiveHeldEntity(player_data.entity, item)
            np.SetActiveHeldEntity(player_data.entity, mActiveItem)
            break
        end
    end
end

local function no_notplayer()
    local ent = LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/local_health/poly.xml")
    EntityAddTag(ent + 1, "ew_notplayer")

    EntityAddComponent2(ent + 1, "LuaComponent", {
        script_item_picked_up = "mods/quant.ew/files/system/potion_mimic/pickup.lua",
        script_throw_item = "mods/quant.ew/files/system/potion_mimic/pickup.lua",
    })

    for _, com in ipairs(EntityGetComponent(ent + 1, "LuaComponent")) do
        if ComponentGetValue2(com, "script_death") == "data/scripts/items/potion_glass_break.lua" then
            EntityRemoveComponent(ent + 1, com)
            break
        end
    end
    for _, com in ipairs(EntityGetComponent(ent + 1, "DamageModelComponent")) do
        EntityRemoveComponent(ent + 1, com)
    end

    polymorph.switch_entity(ent + 1)
end

local function player_died()
    if ctx.my_player.entity == nil then
        return
    end
    if util.is_world_state_entity_like(ctx.my_player.entity) then
        util.log("Err: Current player is world state like.")
        return
    end

    -- Don't show "run ended" as reason of death
    if not ctx.run_ended then
        show_death_message()
    end

    -- This may look like a hack, but it allows to use existing poly machinery to change player entity AND to store the original player for later,
    -- Which is, like, perfect.
    GameAddFlagRun("ew_flag_notplayer_active")
    if ctx.proxy_opt.no_notplayer then
        no_notplayer()
        return
    end
    if ctx.proxy_opt.perma_death then
        remove_inventory()
        local ent = LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/local_health/notplayer/cessation.xml")
        EntityAddTag(ent + 1, "ew_notplayer")
        polymorph.switch_entity(ent + 1)
        GameAddFlagRun("msg_gods_looking")
        GameAddFlagRun("msg_gods_looking2")
        return
    end
    local gold = get_gold()

    rpc.remove_homing(false)
    -- Serialize inventory, perks, and max_hp, we'll need to copy it over to notplayer.
    local item_data = inventory_helper.get_item_data(ctx.my_player)
    remove_inventory()
    local perk_data = perk_fns.get_my_perks()
    local _, max_hp = util.get_ent_health(ctx.my_player.entity)
    local cap = util.get_ent_health_cap(ctx.my_player.entity)

    local ent = LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/local_health/notplayer/poly_effect.xml")
    ctx.my_player.entity = ent + 1
    if ctx.proxy_opt.physics_damage then
        local damage = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "DamageModelComponent")
        ComponentSetValue2(damage, "physics_objects_damage", true)
    end
    do_switch_effect(false)
    EntitySetName(ctx.my_player.entity, ctx.my_id.."?")
    util.set_ent_health(ctx.my_player.entity, {max_hp, max_hp})
    util.set_ent_health_cap(ctx.my_player.entity, cap)
    local iron = LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/local_health/notplayer/iron_stomach.xml")
    EntityAddTag(iron, "kill_on_revive")
    LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/spectate/no_tinker.xml")
    set_cosmetics_locally(ctx.my_id)

    local inv = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "Inventory2Component")
    if inv ~= nil then
        ComponentSetValue2(inv, "mItemHolstered", false)
        ComponentSetValue2(inv, "mActualActiveItem", 0)
        ComponentSetValue2(inv, "mActiveItem", 0)
        local quick
        for _, child in ipairs(EntityGetAllChildren(ctx.my_player.entity) or {}) do
            if EntityGetName(child) == "inventory_quick" then
                quick = child
                break
            end
        end
        for _, child in ipairs(EntityGetAllChildren(quick) or {}) do
            EntitySetComponentsWithTagEnabled(child, "enabled_in_hand", false)
            EntitySetComponentsWithTagEnabled(child, "enabled_in_world", false)
            EntitySetComponentsWithTagEnabled(child, "enabled_in_inventory", true)
        end
    end

    polymorph.switch_entity(ent + 1)

    remove_healthbar_locally()
    inventory_helper.set_item_data(item_data, ctx.my_player, true, false)
    for _, child in ipairs(EntityGetAllChildren(ctx.my_player.entity) or {}) do
        if EntityGetName(child) == "cursor" or EntityGetName(child) == "notcursor" then
            EntitySetComponentIsEnabled(child, EntityGetFirstComponentIncludingDisabled(child, "SpriteComponent"), false)
        end
    end
    reset_cast_state_if_has_any_other_item(ctx.my_player)
    remove_inventory_tags()
    perk_fns.update_perks_for_entity(perk_data, ctx.my_player.entity, allow_notplayer_perk)
    util.set_ent_health(ctx.my_player.entity, {max_hp, max_hp})
    util.set_ent_health_cap(ctx.my_player.entity, cap)
    rpc.add_nickname_change_cursor()
    set_gold(gold)
end

local function do_game_over(message)
    net.proxy_notify_game_over()
    GameRemoveFlagRun("ew_flag_notplayer_active")
    set_camera_free(true, ctx.my_player.entity)

    if not GameHasFlagRun("ending_game_completed") then
        for peer_id, data in pairs(ctx.players) do
            if peer_id ~= ctx.my_id
                    and #(EntityGetAllChildren(data.entity) or {}) ~= 0 then
                local x, y = EntityGetTransform(data.entity)
                LoadRagdoll("mods/quant.ew/files/system/player/tmp/".. peer_id .."_ragdoll.txt", x, y)
            end
        end
    end
    async(function()
        print("Performing do_game_over...")
        if #(EntityGetAllChildren(ctx.my_player.entity) or {}) ~= 0 then
            local ent = end_poly_effect(ctx.my_player.entity)
            if ent ~= nil then
                polymorph.switch_entity(ent)
                wait(1)
                if ctx.my_player.entity ~= nil and EntityGetIsAlive(ctx.my_player.entity) then
                    local damage_model = EntityGetFirstComponent(ctx.my_player.entity, "DamageModelComponent")
                    if damage_model ~= nil then
                        ComponentSetValue2(damage_model, "wait_for_kill_flag_on_death", false)
                        EntityInflictDamage(ctx.my_player.entity, 1000000, "DAMAGE_CURSE", message, "NONE", 0, 0, GameGetWorldStateEntity())
                    end
                end
            end
        end
        ctx.run_ended = true
        print("Running - GameTriggerGameOver")
        GameTriggerGameOver()
        for _, data in pairs(ctx.players) do
            EntityKill(data.entity)
        end
    end)
end

function module.on_local_player_spawn(my_player)
    local damage_model = EntityGetFirstComponentIncludingDisabled(my_player.entity, "DamageModelComponent")
    if damage_model ~= nil then
        ComponentSetValue2(damage_model, "wait_for_kill_flag_on_death", true)
    end
    ctx.my_player.status = { is_alive = true }

    util.ensure_component_present(my_player.entity, "LuaComponent", "ew_player_damage", {
        script_damage_received = "mods/quant.ew/files/system/local_health/grab_damage_message.lua"
    })
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

    if ctx.proxy_opt.no_notplayer and notplayer_active then
        local x, y = EntityGetTransform(ctx.my_player.entity)
        for _, ent in ipairs(EntityGetInRadiusWithTag(x, y, 14, "drillable")) do
            if EntityGetFilename(ent) == "data/entities/items/pickup/heart_fullhp_temple.xml" then
                GameRemoveFlagRun("ew_flag_notplayer_active")
                EntityKill(ent)
                ctx.my_player.entity = end_poly_effect(ctx.my_player.entity)
                remove_stuff()
                polymorph.switch_entity(ctx.my_player.entity)
                spectate.disable_throwing(false, ctx.my_player.entity)
                reduce_hp()
                break
            end
        end
    end

    local hp_new, max_hp_new, has_hp = util.get_ent_health(ctx.my_player.entity)
    if not ctx.my_player.currently_polymorphed and has_hp then
        if hp_new <= 0 then
            -- Restore the player back to small amount of hp.
            local new_hp = 3 * max_hp_new / 20
            if ctx.proxy_opt.no_notplayer then
                new_hp = new_hp * 3
            end
            local final_hp = math.max(new_hp, math.min(2/5, max_hp_new))
            util.set_ent_health(ctx.my_player.entity, {final_hp, max_hp_new})
            player_died()
        end
    end
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
            print("Triggering a game over")
            rpc.trigger_game_over("No players are alive")
        end
    end
end

function module.on_new_player_seen(new_playerdata, player_count)
end

function module.on_client_spawned(peer_id, playerdata)
    playerdata.status = { is_alive = true }
    EntityAddComponent2(playerdata.entity, "LuaComponent", {_tags="ew_immortal", script_damage_about_to_be_received = "mods/quant.ew/files/resource/cbs/immortal.lua"})
    if ctx.is_host then
        EntityAddComponent2(playerdata.entity, "LuaComponent", {script_damage_received = "mods/quant.ew/files/system/damage/cbs/send_damage_to_client.lua"})
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

rpc.opts_reliable()
function rpc.loss_hp()
    local p = 100 - ctx.proxy_opt.health_lost_on_revive
    local hp, max_hp = util.get_ent_health(ctx.my_player.entity)
    util.set_ent_health(ctx.my_player.entity, {(hp * p) / 100, (max_hp * p) / 100})
end

-- Provides health capability
ctx.cap.health = {
    health = module.health,
    max_health = module.max_health,
    set_health = module.set_health,
    set_max_health = module.set_max_health,
    inflict_damage = module.inflict_damage,
    do_game_over = function(message) rpc.trigger_game_over(message) end,
    on_poly_death = function()
        local notplayer_active = GameHasFlagRun("ew_flag_notplayer_active")
        if notplayer_active then
            if GameHasFlagRun("ending_game_completed") and not GameHasFlagRun("ew_kill_player") then
                return
            end
            if ctx.proxy_opt.no_notplayer then
                polymorph.switch_entity(end_poly_effect(ctx.my_player.entity))
                async(function()
                    wait(1)
                    player_died()
                end)
                return
            end
            local gold = get_gold()
            rpc.remove_homing(true)
            local item_data = inventory_helper.get_item_data(ctx.my_player)
            remove_inventory()
            GameRemoveFlagRun("ew_flag_notplayer_active")
            ctx.my_player.entity = end_poly_effect(ctx.my_player.entity)
            remove_stuff()
            inventory_helper.set_item_data(item_data, ctx.my_player, true, false)
            remove_inventory_tags()
            local controls = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "ControlsComponent")
            if controls ~= nil then
                ComponentSetValue2(controls, "enabled", true)
            end
            if GameHasFlagRun("ew_kill_player") then
                GameRemoveFlagRun("ew_kill_player")
                polymorph.switch_entity(ctx.my_player.entity)
                async(function()
                    wait(1)
                    if GameHasFlagRun("ending_game_completed") then
                        EntityInflictDamage(ctx.my_player.entity, -1000000, "DAMAGE_HEALING", "", "NONE", 0, 0, GameGetWorldStateEntity())
                    else
                        EntityInflictDamage(ctx.my_player.entity, 1000000, "DAMAGE_CURSE", "", "NONE", 0, 0, GameGetWorldStateEntity())
                    end
                end)
            else
                do_switch_effect(true)
                polymorph.switch_entity(ctx.my_player.entity)
            end
            reduce_hp()
            spectate.disable_throwing(false, ctx.my_player.entity)
            set_gold(gold)
        else
            polymorph.switch_entity(end_poly_effect(ctx.my_player.entity))
            local _, max_hp_new, has_hp = util.get_ent_health(ctx.my_player.entity)
            if not ctx.my_player.currently_polymorphed and has_hp then
                -- Restore the player back to small amount of hp.
                local new_hp = 3 * max_hp_new / 20
                local final_hp = math.max(new_hp, math.min(2/5, max_hp_new))
                util.set_ent_health(ctx.my_player.entity, {final_hp, max_hp_new})
            end
            async(function()
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
util.add_cross_call("ew_ds_client_damaged", rpc.melee_damage_client)

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