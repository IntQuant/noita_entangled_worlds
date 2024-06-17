local util = dofile_once("mods/quant.ew/files/src/util.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local np = require("noitapatcher")

local rpc = net.new_rpc_namespace()

local enemy_sync = {}

local dead_entities = {}
local confirmed_kills = {}

np.CrossCallAdd("ew_es_death_notify", function(enemy_id, responsible_id)
    local player_data = player_fns.get_player_data_by_local_entity_id(responsible_id)
    local responsible = nil
    if player_data ~= nil then
        responsible = player_data.peer_id
    else
        responsible = responsible_id
    end
    table.insert(dead_entities, {enemy_id, responsible})
end)

local function world_exists_for(entity)
    local x, y = EntityGetFirstHitboxCenter(entity)
    local w, h = 5, 5 -- TODO
    w = w * 0.5
    h = h * 0.5
    return DoesWorldExistAt(x - w, y - h, x + w, y + h)
end

local function table_extend(to, from)
    for _, e in ipairs(from) do
        to[#to+1] = e
    end
end

local function table_extend_filtered(to, from, filter)
    for _, e in ipairs(from) do
        if filter(e) then
            to[#to+1] = e
        end
    end
end

local function get_sync_entities()
    local entities = {} -- TODO maybe only sync those close to players?
    table_extend_filtered(entities, EntityGetWithTag("enemy"), function (ent)
        return not (EntityHasTag(ent, "ew_no_enemy_sync"))
    end)
    -- table_extend_filtered(entities, EntityGetWithTag("projectile"), function (ent)
    --     return not (EntityHasTag(ent, "ew_no_enemy_sync") or EntityHasTag(ent, "projectile_player"))
    -- end)
    return entities
end

function enemy_sync.host_upload_entities()
    local entities = get_sync_entities()
    local enemy_data_list = {}
    for i, enemy_id in ipairs(entities) do
        if not world_exists_for(enemy_id) then
            goto continue
        end
        local filename = EntityGetFilename(enemy_id)
        local x, y = EntityGetTransform(enemy_id)
        local character_data = EntityGetFirstComponentIncludingDisabled(enemy_id, "VelocityComponent")
        local vx, vy = 0, 0
        if character_data ~= nil then
            vx, vy = ComponentGetValue2(character_data, "mVelocity")
        end
        local ai_component = EntityGetFirstComponentIncludingDisabled(enemy_id, "AnimalAIComponent")
        if ai_component ~= 0 and ai_component ~= nil then
            ComponentSetValue2(ai_component, "max_distance_to_cam_to_start_hunting", math.pow(2, 29))
        end
        local hp, max_hp, has_hp = util.get_ent_health(enemy_id)

        if has_hp then
            util.ensure_component_present(enemy_id, "LuaComponent", "ew_death_notify", {
                script_death = "mods/quant.ew/files/cbs/death_notify.lua"
            })
        end

        table.insert(enemy_data_list, {enemy_id, filename, x, y, vx, vy, hp, max_hp})
        ::continue::
    end

    rpc.handle_enemy_data(enemy_data_list)
    if #dead_entities > 0 then
        rpc.handle_death_data(dead_entities)
    end
    dead_entities = {}
end

function enemy_sync.client_cleanup()
    local entities = get_sync_entities()
    for i, enemy_id in ipairs(entities) do
        if not EntityHasTag(enemy_id, "ew_replicated") then
            print("Despawning unreplicated "..enemy_id)
            EntityKill(enemy_id)
        end
    end
    local frame = GameGetFrameNum()
    for remote_id, enemy_data in pairs(ctx.entity_by_remote_id) do
        if frame - enemy_data.frame > 60*1 then
            print("Despawning stale "..remote_id.." "..enemy_data.id)
            EntityKill(enemy_data.id)
            ctx.entity_by_remote_id[remote_id] = nil
        end
    end
end

function enemy_sync.on_world_update_host()
    if GameGetFrameNum() % 2 == 1 then
        enemy_sync.host_upload_entities()
    end
end

function enemy_sync.on_world_update_client()
    if GameGetFrameNum() % 20 == 1 then
        enemy_sync.client_cleanup()
    end
end

rpc.opts_reliable()
function rpc.handle_death_data(death_data)
    for _, remote_data in ipairs(death_data) do
        local remote_id = remote_data[1]
        if confirmed_kills[remote_id] then
            GamePrint("Remote id has been killed already..?")
            goto continue
        end
        confirmed_kills[remote_id] = true
        local responsible_entity = 0
        local peer_data = player_fns.peer_get_player_data(remote_data[2], true)
        if peer_data ~= nil then
            responsible_entity = peer_data.entity
        elseif ctx.entity_by_remote_id[remote_data[2]] ~= nil then
            responsible_entity = ctx.entity_by_remote_id[remote_data[2]]
        end

        local enemy_data = ctx.entity_by_remote_id[remote_id]
        if enemy_data ~= nil and EntityGetIsAlive(enemy_data.id) then
            local enemy_id = enemy_data.id
            local immortal = EntityGetFirstComponentIncludingDisabled(enemy_id, "LuaComponent", "ew_immortal")
            if immortal ~= 0 then
                EntityRemoveComponent(enemy_id, immortal)
            end
            local protection_component_id = GameGetGameEffect(enemy_id, "PROTECTION_ALL")
            if protection_component_id ~= 0 then
                EntitySetComponentIsEnabled(enemy_id, protection_component_id, false)
            end

            local damage_component = EntityGetFirstComponentIncludingDisabled(enemy_id, "DamageModelComponent")
            if damage_component and damage_component ~= 0 then
                ComponentSetValue2(damage_component, "wait_for_kill_flag_on_death", false)
            end

            local current_hp = util.get_ent_health(enemy_id)
            local dmg = current_hp
            if dmg > 0 then
                EntityInflictDamage(enemy_id, dmg+0.1, "DAMAGE_CURSE", "", "NONE", 0, 0, responsible_entity)
            end
            EntityInflictDamage(enemy_id, 1000000000, "DAMAGE_CURSE", "", "NONE", 0, 0, responsible_entity) -- Just to be sure
            EntityKill(enemy_id)
        end
        ::continue::
    end
end

function rpc.handle_enemy_data(enemy_data)
    -- GamePrint("Got enemy data")
    for _, enemy_info_raw in ipairs(enemy_data) do
        local remote_enemy_id = enemy_info_raw[1]
        local filename = enemy_info_raw[2]
        local x = enemy_info_raw[3]
        local y = enemy_info_raw[4]
        local vx = enemy_info_raw[5]
        local vy = enemy_info_raw[6]
        local hp = enemy_info_raw[7]
        local max_hp = enemy_info_raw[8]
        local has_died = filename == nil

        local frame = GameGetFrameNum()
        
        if confirmed_kills[remote_enemy_id] then
            goto continue
        end

        if ctx.entity_by_remote_id[remote_enemy_id] ~= nil and not EntityGetIsAlive(ctx.entity_by_remote_id[remote_enemy_id].id) then
            ctx.entity_by_remote_id[remote_enemy_id] = nil
        end
            
        if ctx.entity_by_remote_id[remote_enemy_id] == nil then
            if filename == nil then
                goto continue
            end
            local enemy_id = EntityLoad(filename, x, y)
            EntityAddTag(enemy_id, "ew_replicated")
            EntityAddTag(enemy_id, "polymorphable_NOT")
            EntityAddComponent2(enemy_id, "LuaComponent", {_tags="ew_immortal", script_damage_about_to_be_received = "mods/quant.ew/files/cbs/immortal.lua"})
            local damage_component = EntityGetFirstComponentIncludingDisabled(enemy_id, "DamageModelComponent")
            if damage_component and damage_component ~= 0 then
                ComponentSetValue2(damage_component, "wait_for_kill_flag_on_death", true)
            end
            for _, name in ipairs({"AnimalAIComponent", "PhysicsAIComponent", "PhysicsBodyComponent"}) do
                local ai_component = EntityGetFirstComponentIncludingDisabled(enemy_id, name)
                if ai_component ~= 0 then
                    EntityRemoveComponent(enemy_id, ai_component)
                end
            end
            ctx.entity_by_remote_id[remote_enemy_id] = {id = enemy_id, frame = frame}
        end

        local enemy_data = ctx.entity_by_remote_id[remote_enemy_id]
        enemy_data.frame = frame
        local enemy_id = enemy_data.id

        if not has_died then
            local character_data = EntityGetFirstComponentIncludingDisabled(enemy_id, "CharacterDataComponent")
            if character_data ~= nil then
                ComponentSetValue2(character_data, "mVelocity", vx, vy)
            end
            local velocity_data = EntityGetFirstComponentIncludingDisabled(enemy_id, "VelocityComponent")
            if velocity_data ~= nil then
                ComponentSetValue2(velocity_data, "mVelocity", vx, vy)
            end
            EntitySetTransform(enemy_id, x, y)
        end
        local current_hp = util.get_ent_health(enemy_id)
        local dmg = current_hp-hp
        if dmg > 0 then
            -- Make sure the enemy doesn't die from the next EntityInflictDamage.
            util.set_ent_health(enemy_id, {dmg*2, dmg*2})
            -- Deal damage, so that game displays damage numbers.
            EntityInflictDamage(enemy_id, dmg, "DAMAGE_CURSE", "", "NONE", 0, 0, GameGetWorldStateEntity())
        end
        util.set_ent_health(enemy_id, {hp, max_hp})
        ::continue::
    end
end

function enemy_sync.on_projectile_fired(shooter_id, projectile_id, initial_rng, position_x, position_y, target_x, target_y, send_message,
    unknown1, multicast_index, unknown3)
    local not_a_player = not EntityHasTag(shooter_id, "ew_no_enemy_sync") and not EntityHasTag(shooter_id, "player_unit") and not EntityHasTag(shooter_id, "ew_client")
    if not_a_player and ctx.is_host then
        local projectileComponent = EntityGetFirstComponentIncludingDisabled(projectile_id, "ProjectileComponent")
        local entity_that_shot    = ComponentGetValue2(projectileComponent, "mEntityThatShot")
        if entity_that_shot == 0 then
            rpc.replicate_projectile(np.SerializeEntity(projectile_id), position_x, position_y, target_x, target_y, shooter_id, initial_rng)
        end
    end
end

rpc.opts_reliable()
function rpc.replicate_projectile(seri_ent, position_x, position_y, target_x, target_y, remote_source_ent, rng)
    np.SetProjectileSpreadRNG(rng)
    local source_ent = ctx.entity_by_remote_id[remote_source_ent].id
    local ent = EntityCreateNew()
    np.DeserializeEntity(ent, seri_ent)
    GameShootProjectile(source_ent, position_x, position_y, target_x, target_y, ent)
end


return enemy_sync