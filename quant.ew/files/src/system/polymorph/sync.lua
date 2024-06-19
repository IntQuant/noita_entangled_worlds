local util = dofile_once("mods/quant.ew/files/src/util.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local np = require("noitapatcher")

local rpc = net.new_rpc_namespace()

local module = {}

local function entity_changed()
    local currently_polymorphed = not EntityHasTag(ctx.my_player.entity, "player_unit")

    ctx.my_player.currently_polymorphed = currently_polymorphed
    if currently_polymorphed then
        local damage_model = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "DamageModelComponent")
        ComponentSetValue2(damage_model, "wait_for_kill_flag_on_death", true)
        rpc.change_entity(np.SerializeEntity(ctx.my_player.entity))
    else
        rpc.change_entity(nil)
    end
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
        local hp = util.get_ent_health(ctx.my_player.entity)
        if hp <= 0 and not gameover_requested then
            ctx.cap.health.do_game_over()
            gameover_requested = true
        end
    end
end

function module.on_world_update_post()
    local ent = np.GetPlayerEntity()
    if ent ~= ctx.my_player.entity then
        player_fns.replace_player_entity(ent, ctx.my_player)
        EntityAddTag(ent, "ew_no_enemy_sync")
        entity_changed()
    end
end

function module.on_projectile_fired(shooter_id, projectile_id, initial_rng, position_x, position_y, target_x, target_y, send_message,
    unknown1, multicast_index, unknown3)
    if ctx.my_player.currently_polymorphed and shooter_id == ctx.my_player.entity then
        local projectileComponent = EntityGetFirstComponentIncludingDisabled(projectile_id, "ProjectileComponent")
        local entity_that_shot    = ComponentGetValue2(projectileComponent, "mEntityThatShot")
        if entity_that_shot == 0 then
            rpc.replicate_projectile(np.SerializeEntity(projectile_id), position_x, position_y, target_x, target_y)
        end
        EntityAddTag(projectile_id, "ew_no_enemy_sync")
    end
end

rpc.opts_reliable()
function rpc.replicate_projectile(seri_ent, position_x, position_y, target_x, target_y)
    local ent = EntityCreateNew()
    np.DeserializeEntity(ent, seri_ent)
    EntityAddTag(ent, "ew_no_enemy_sync")
    GameShootProjectile(ctx.rpc_player_data.entity, position_x, position_y, target_x, target_y, ent)
end

rpc.opts_reliable()
function rpc.change_entity(seri_ent)
    if seri_ent ~= nil then
        local ent = EntityCreateNew()
        np.DeserializeEntity(ent, seri_ent)
        EntityAddComponent2(ent, "LuaComponent", {script_damage_about_to_be_received = "mods/quant.ew/files/cbs/immortal.lua"})
        EntityAddTag(ent, "ew_no_enemy_sync")
        
        -- Remove all poly-like effects to prevent spawn of another player character when it runs out
        remove_all_effects(ent)

        local controls = EntityGetFirstComponentIncludingDisabled(ent, "ControlsComponent")
        if controls then
            EntitySetComponentIsEnabled(ent, controls, false)
        end
        
        EntityKill(ctx.rpc_player_data.entity)
        player_fns.replace_player_entity(ent, ctx.rpc_player_data)
    else
        EntityKill(ctx.rpc_player_data.entity)
        player_fns.replace_player_entity(nil, ctx.rpc_player_data)
    end
end

return module
