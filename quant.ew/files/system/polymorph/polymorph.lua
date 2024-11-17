local util = dofile_once("mods/quant.ew/files/core/util.lua")
local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/core/player_fns.lua")
local np = require("noitapatcher")

local rpc = net.new_rpc_namespace()

local module = {}

local function entity_changed()
    local currently_polymorphed = EntityGetName(ctx.my_player.entity) ~= "DEBUG_NAME:player"
    print(tostring(currently_polymorphed))
    ctx.my_player.currently_polymorphed = currently_polymorphed
    if currently_polymorphed then
        local damage_model = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "DamageModelComponent")
        if damage_model ~= nil then
            ComponentSetValue2(damage_model, "wait_for_kill_flag_on_death", true)
        end
        if ctx.my_player.entity == GameGetWorldStateEntity() then
            print("Player entity is equal to WSE, skipping...")
            return
        end
        for _, com in ipairs(EntityGetComponent(ctx.my_player.entity, "CollisionTriggerComponent") or {}) do
            if ComponentGetValue2(com, "destroy_this_entity_when_triggered") then
                ComponentSetValue2(com, "destroy_this_entity_when_triggered", false)
                ComponentSetValue2(com, "remove_component_when_triggered", true)
            end
        end

        rpc.change_entity({data = util.serialize_entity(ctx.my_player.entity)})
    else
        rpc.change_entity(nil)
        local controls = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "ControlsComponent")
        ComponentSetValue2(controls, "enabled", true)
    end
    ctx.hook.on_local_player_polymorphed(currently_polymorphed)
end

function module.on_local_player_spawn()
    local currently_polymorphed = EntityGetName(ctx.my_player.entity) ~= "DEBUG_NAME:player"
    ctx.my_player.currently_polymorphed = currently_polymorphed
end

function module.on_should_send_updates()
    entity_changed()
end

local function get_ent_effects(entity)
    local list = {}
    for _, ent in ipairs(EntityGetAllChildren(entity) or {}) do
        local com = EntityGetFirstComponentIncludingDisabled(ent, "GameEffectComponent")
        if com ~= nil then
            table.insert(list, ent)
        end
    end
    return list
end

local function remove_all_effects(entity)
    local effects = get_ent_effects(entity)
    for _, effect in ipairs(effects) do
        EntityKill(effect)
    end
end

local gameover_requested = false

function module.on_world_update()
    if ctx.my_player.currently_polymorphed then
        local hp, _, has_hp_component = util.get_ent_health(ctx.my_player.entity)
        -- Added a check for having damage model component at all, as entity can't die from lack of health in that case.
        if has_hp_component and hp <= 0 and not gameover_requested then
            ctx.cap.health.on_poly_death()
            gameover_requested = true
        end
    else
        gameover_requested = false
    end
end

function module.switch_entity(ent)
    player_fns.replace_player_entity(ent, ctx.my_player)
    EntityAddTag(ent, "ew_no_enemy_sync")
    entity_changed()
end

function module.on_world_update_post()
    local ent = np.GetPlayerEntity()
    if ent ~= nil and ent ~= ctx.my_player.entity then
        if EntityHasTag(ent, "mimic_potion") then
            local effect
            for _, child in ipairs(EntityGetAllChildren(ctx.my_player.entity) or {}) do
                local com = EntityGetFirstComponentIncludingDisabled(child, "GameEffectComponent")
                if com ~= nil then
                    local effect_name = ComponentGetValue2(com, "effect")
                    if effect_name == "POLYMORPH" or effect_name == "POLYMORPH_RANDOM"
                            or effect_name == "POLYMORPH_CESSATION" or effect_name == "POLYMORPH_UNSTABLE" then
                        effect = com
                        break
                    end
                end
            end
            if effect ~= nil then
                local frames = ComponentGetValue2(effect, "frames")
                if frames < 1200 and frames > 0 then
                    ComponentSetValue2(effect, "frames", 1200)
                end
            end

            EntityAddComponent2(ent, "LuaComponent", {
                script_item_picked_up = "mods/quant.ew/files/system/potion_mimic/pickup.lua",
                script_throw_item = "mods/quant.ew/files/system/potion_mimic/pickup.lua",
            })
        end
        module.switch_entity(ent)
        if ctx.proxy_opt.game_mode == "local_health" then
            util.ensure_component_present(ent, "LuaComponent", "ew_player_damage", {
                script_damage_received = "mods/quant.ew/files/system/local_health/grab_damage_message.lua"
            })
        end
    end
end

function module.on_projectile_fired(shooter_id, projectile_id, initial_rng, position_x, position_y, target_x, target_y, send_message,
    unknown1, multicast_index, unknown3)
    -- Do not sync notplayer's projectiles extra time.
    if ctx.my_player.currently_polymorphed and shooter_id == ctx.my_player.entity and not EntityHasTag(shooter_id, "player_unit") then
        local projectileComponent = EntityGetFirstComponentIncludingDisabled(projectile_id, "ProjectileComponent")
        if projectileComponent ~= nil then
            local entity_that_shot    = ComponentGetValue2(projectileComponent, "mEntityThatShot")
            if entity_that_shot == 0 then
                rpc.replicate_projectile(util.serialize_entity(projectile_id), position_x, position_y, target_x, target_y)
            end
            EntityAddTag(projectile_id, "ew_no_enemy_sync")
        end
    end
end

rpc.opts_reliable()
function rpc.replicate_projectile(seri_ent, position_x, position_y, target_x, target_y)
    local ent = util.deserialize_entity(seri_ent)
    EntityAddTag(ent, "ew_no_enemy_sync")
    GameShootProjectile(ctx.rpc_player_data.entity, position_x, position_y, target_x, target_y, ent)
end

local function apply_seri_ent(player_data, seri_ent)
    if seri_ent ~= nil then
        local ent = util.deserialize_entity(seri_ent.data)
        EntityAddTag(ent, "ew_no_enemy_sync")
        EntityAddTag(ent, "ew_client")

        EntityAddComponent2(ent, "LuaComponent", {script_damage_about_to_be_received = "mods/quant.ew/files/resource/cbs/immortal.lua"})

        -- Remove all poly-like effects to prevent spawn of another player character when it runs out
        remove_all_effects(ent)

        local controls = EntityGetFirstComponentIncludingDisabled(ent, "ControlsComponent")
        if controls then
            ComponentSetValue2(controls, "enabled", false)
        end
        local inv = EntityGetFirstComponentIncludingDisabled(ent, "InventoryGuiComponent")
        if inv then
            EntityRemoveComponent(ent, inv)
        end
        for _, comp in ipairs(EntityGetAllComponents(ent)) do
            if ComponentHasTag(comp, "ew_remove_on_send") then
                EntityRemoveComponent(ent, comp)
            end
        end
        util.set_ent_firing_blocked(ent, true)
        EntityRemoveTag(ent, "player_unit")
        EntityRemoveTag(ent, "teleportable")

        EntitySetName(ent, player_data.peer_id.."?")

        EntityKill(player_data.entity)
        player_fns.replace_player_entity(ent, player_data)
        player_data.currently_polymorphed = true
        ctx.hook.on_client_polymorphed(ctx.rpc_peer_id, player_data)
    else
        if player_data.currently_polymorphed then
            EntityKill(ctx.rpc_player_data.entity)
            player_fns.replace_player_entity(nil, player_data)
            player_data.currently_polymorphed = false
        else
            print("Player is already not polymorphed, not doing anything")
        end
    end
end

local last_seri_ent = {}

rpc.opts_reliable()
function rpc.change_entity(seri_ent)
    last_seri_ent[ctx.rpc_peer_id] = seri_ent
    apply_seri_ent(ctx.rpc_player_data, seri_ent)
end

function module.on_client_spawned(peer_id, player_data)
    player_data.currently_polymorphed = false
    apply_seri_ent(player_data, last_seri_ent[peer_id])
end

return module