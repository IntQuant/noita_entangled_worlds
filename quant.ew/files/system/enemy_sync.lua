local util = dofile_once("mods/quant.ew/files/core/util.lua")
local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/core/player_fns.lua")
local np = require("noitapatcher")

local ffi = require("ffi")
local rpc = net.new_rpc_namespace()

local EnemyData = util.make_type({
    u32 = {"enemy_id"},
    f32 = {"x", "y", "vx", "vy"},
})

-- Variant of EnemyData for when we don't have any motion (or no VelocityComponent).
local EnemyDataNoMotion = util.make_type({
    u32 = {"enemy_id"},
    f32 = {"x", "y"}
})

local HpData = util.make_type({
    u32 = {"enemy_id"},
    f32 = {"hp", "max_hp"}
})

local FULL_TURN = math.pi * 2

local PhysData = util.make_type({
    f32 = {"x", "y", "vx", "vy", "vr"},
    -- We should be able to cram rotation into 1 byte.
    u8 = {"r"}
})

-- Variant of PhysData for when we don't have any motion.
local PhysDataNoMotion = util.make_type({
    f32 = {"x", "y"},
    -- We should be able to cram rotation into 1 byte.
    u8 = {"r"}
})

local enemy_sync = {}

local dead_entities = {}
-- TODO this basically never happens, doesn't seem that useful anymore. Perhaps should be removed to conserve memory.
local confirmed_kills = {}

local spawned_by_us = {}

-- HACK
local times_spawned_last_minute = {}

local DISTANCE_LIMIT = 128*5

for filename, _ in pairs(constants.phys_sync_allowed) do
    util.add_tag_to(filename, "prop_physics")
    -- Idk it just causes the minecart to not appear at all.
    -- util.replace_text_in(filename, 'kill_entity_after_initialized="1"', 'kill_entity_after_initialized="0"')
end

np.CrossCallAdd("ew_es_death_notify", function(enemy_id, responsible_id)
    local player_data = player_fns.get_player_data_by_local_entity_id(responsible_id)
    local responsible
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

local function serialize_phys_component(phys_component)
    local px, py, pr, pvx, pvy, pvr = np.PhysBodyGetTransform(phys_component)
    if math.abs(pvx) < 0.01 and math.abs(pvy) < 0.01 and math.abs(pvr) < 0.01 then
        return PhysDataNoMotion {
            x = px,
            y = py,
            r = math.floor((pr % FULL_TURN) / FULL_TURN * 255),
        }
    else
        return PhysData {
            x = px,
            y = py,
            r = math.floor((pr % FULL_TURN) / FULL_TURN * 255),
            vx = pvx,
            vy = pvy,
            vr = pvr,
        }
    end
end

local function deserialize_phys_component(phys_component, phys_info)
    if ffi.typeof(phys_info) == PhysDataNoMotion then
        np.PhysBodySetTransform(phys_component, phys_info.x, phys_info.y, phys_info.r / 255 * FULL_TURN, 0, 0, 0)
    else
        np.PhysBodySetTransform(phys_component, phys_info.x, phys_info.y, phys_info.r / 255 * FULL_TURN, phys_info.vx, phys_info.vy, phys_info.vr)
    end
end

local function get_sync_entities(return_all)
    local entities = {}
    table_extend_filtered(entities, EntityGetWithTag("enemy"), function (ent)
        return not (EntityHasTag(ent, "ew_no_enemy_sync"))
    end)
    table_extend(entities, EntityGetWithTag("ew_enemy_sync_extra"))
    table_extend_filtered(entities, EntityGetWithTag("prop_physics"), function (ent)
        return constants.phys_sync_allowed[EntityGetFilename(ent)]
    end)

    local skipped_counter = 0
    local entities2 = {}
    if return_all then
        entities2 = entities
    else
        table_extend_filtered(entities2, entities, function(ent)
            local x, y = EntityGetTransform(ent)
            local has_anyone = #EntityGetInRadiusWithTag(x, y, DISTANCE_LIMIT, "ew_peer") > 0
            if not has_anyone then
                skipped_counter = skipped_counter + 1
            end
            return has_anyone
        end)
    end
    -- GamePrint("skipped "..skipped_counter.." out of "..#entities)

    return entities2
end

function enemy_sync.host_upload_entities()
    local entities = get_sync_entities()
    local enemy_data_list = {}
    for i, enemy_id in ipairs(entities) do
        if not world_exists_for(enemy_id) then
            goto continue
        end
        local filename = EntityGetFilename(enemy_id)
        filename = constants.interned_filename_to_index[filename] or filename

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

        local phys_info = {}
        local phys_info_2 = {}
        -- Some things (like physics object) don't react well to making their entities ephemerial.
        local not_ephemerial = false

        for _, phys_component in ipairs(EntityGetComponent(enemy_id, "PhysicsBodyComponent") or {}) do
            if phys_component ~= nil and phys_component ~= 0 then
                not_ephemerial = true
                table.insert(phys_info, serialize_phys_component(phys_component))
            end
        end

        for _, phys_component in ipairs(EntityGetComponent(enemy_id, "PhysicsBody2Component") or {}) do
            if phys_component ~= nil and phys_component ~= 0 then
                not_ephemerial = true
                local initialized = ComponentGetValue2(phys_component, "mInitialized")
                if initialized then
                    table.insert(phys_info_2, serialize_phys_component(phys_component))
                else
                    table.insert(phys_info_2, nil)
                end
            end
        end

        if has_hp then
            util.ensure_component_present(enemy_id, "LuaComponent", "ew_death_notify", {
                script_death = "mods/quant.ew/files/resource/cbs/death_notify.lua"
            })
        end

        -- TODO: figure out how to sync those.
        -- local laser_sight_data = nil
        -- local laser_sight = EntityGetFirstComponentIncludingDisabled(enemy_id, "SpriteComponent", "laser_sight")
        -- if laser_sight ~= nil and laser_sight ~= 0 then
        --     -- local x, y, r =
        -- end

        local en_data
        if math.abs(vx) < 0.01 and math.abs(vy) < 0.01 then
            en_data= EnemyDataNoMotion {
                enemy_id = enemy_id,
                x = x,
                y = y,
            }
        else
            en_data= EnemyData {
                enemy_id = enemy_id,
                x = x,
                y = y,
                vx = vx,
                vy = vy,
            }
        end

        table.insert(enemy_data_list, {filename, en_data, not_ephemerial, phys_info, phys_info_2})
        ::continue::
    end

    -- local estimate = net.estimate_rpc_size(enemy_data_list)
    -- GamePrint(#enemy_data_list.." "..net.estimate_rpc_size(enemy_data_list).." "..(estimate*30))

    rpc.handle_enemy_data(enemy_data_list)
    if #dead_entities > 0 then
        rpc.handle_death_data(dead_entities)
    end
    dead_entities = {}
end

local function host_upload_health()
    local entities = get_sync_entities()

    local enemy_health_list = {}
    for i, enemy_id in ipairs(entities) do
        if not world_exists_for(enemy_id) then
            goto continue
        end

        local hp, max_hp, has_hp = util.get_ent_health(enemy_id)

        if has_hp then
            table.insert(enemy_health_list, HpData {
                enemy_id = enemy_id,
                hp = hp,
                max_hp = max_hp,
            })
        end

        ::continue::
    end

    if #enemy_health_list > 0 then
        rpc.handle_enemy_health(enemy_health_list)
    end
end

function enemy_sync.client_cleanup()
    local entities = get_sync_entities(true)
    for i, enemy_id in ipairs(entities) do
        if not EntityHasTag(enemy_id, "ew_replicated") then
            local filename = EntityGetFilename(enemy_id)
            print("Despawning unreplicated "..enemy_id.." "..filename)
            EntityKill(enemy_id)
        elseif not spawned_by_us[enemy_id] then
            local filename = EntityGetFilename(enemy_id)
            print("Despawning persisted "..enemy_id.." "..filename)
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
    if GameGetFrameNum() % ctx.proxy_opt.enemy_sync_interval == 1 then
        enemy_sync.host_upload_entities()
    end
    if GameGetFrameNum() % 10 == 5 then
        host_upload_health()
    end
end

function enemy_sync.on_world_update_client()
    if GameGetFrameNum() % 20 == 1 then
        enemy_sync.client_cleanup()
    end
    if GameGetFrameNum() % (60*60) == 1 then
        times_spawned_last_minute = {}
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

            -- Enable explosion back
            local expl_component = EntityGetFirstComponent(enemy_id, "ExplodeOnDamageComponent")
            if expl_component ~= nil and expl_component ~= 0 then
                ComponentSetValue2(expl_component, "explode_on_death_percent", 1)
            end

            local current_hp = util.get_ent_health(enemy_id)
            local dmg = current_hp
            if dmg > 0 then
                EntityInflictDamage(enemy_id, dmg+0.1, "DAMAGE_CURSE", "", "NONE", 0, 0, responsible_entity)
            end

            EntityInflictDamage(enemy_id, 1000000000, "DAMAGE_CURSE", "", "NONE", 0, 0, responsible_entity) -- Just to be sure
            local parent = EntityGetParent(enemy_id)
            if parent ~= nil then
                EntityKill(parent)
            end
            EntityKill(enemy_id)
        end
        ::continue::
    end
end

function rpc.handle_enemy_data(enemy_data)
    -- GamePrint("Got enemy data: "..#enemy_data)
    for _, enemy_info_raw in ipairs(enemy_data) do
        local filename = enemy_info_raw[1]
        filename = constants.interned_index_to_filename[filename] or filename

        local en_data = enemy_info_raw[2]
        local remote_enemy_id = en_data.enemy_id

        local x = en_data.x
        local y = en_data.y
        local vx = 0
        local vy = 0
        if ffi.typeof(en_data) == EnemyData then
            vx = en_data.vx
            vy = en_data.vy
        end
        local not_ephemerial = enemy_info_raw[3]
        local phys_infos = enemy_info_raw[4]
        local phys_infos_2 = enemy_info_raw[5]
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
            times_spawned_last_minute[remote_enemy_id] = (times_spawned_last_minute[remote_enemy_id] or 0) + 1
            if times_spawned_last_minute[remote_enemy_id] > 5 then
                if times_spawned_last_minute[remote_enemy_id] == 6 then
                    print("Entity has been spawned again more than 5 times in last minute, skipping "..filename)
                end
                goto continue
            end
            local enemy_id
            if not_ephemerial then
                enemy_id = EntityLoad(filename, x, y)
            else
                enemy_id = util.load_ephemerial(filename, x, y)
            end
            spawned_by_us[enemy_id] = true
            EntityAddTag(enemy_id, "ew_replicated")
            EntityAddTag(enemy_id, "polymorphable_NOT")
            EntityAddComponent2(enemy_id, "LuaComponent", {_tags="ew_immortal", script_damage_about_to_be_received = "mods/quant.ew/files/resource/cbs/immortal.lua"})
            local damage_component = EntityGetFirstComponentIncludingDisabled(enemy_id, "DamageModelComponent")
            if damage_component and damage_component ~= 0 then
                ComponentSetValue2(damage_component, "wait_for_kill_flag_on_death", true)
            end
            for _, name in ipairs({"AnimalAIComponent", "PhysicsAIComponent", "CameraBoundComponent"}) do
                local ai_component = EntityGetFirstComponentIncludingDisabled(enemy_id, name)
                if ai_component ~= 0 then
                    EntityRemoveComponent(enemy_id, ai_component)
                end
            end
            ctx.entity_by_remote_id[remote_enemy_id] = {id = enemy_id, frame = frame}

            for _, phys_component in ipairs(EntityGetComponent(enemy_id, "PhysicsBody2Component") or {}) do
                if phys_component ~= nil and phys_component ~= 0 then
                    ComponentSetValue2(phys_component, "destroy_body_if_entity_destroyed", true)
                end
            end
            -- Make sure stuff doesn't decide to explode on clients by itself.
            local expl_component = EntityGetFirstComponent(enemy_id, "ExplodeOnDamageComponent")
            if expl_component ~= nil and expl_component ~= 0 then
                ComponentSetValue2(expl_component, "explode_on_damage_percent", 0)
                ComponentSetValue2(expl_component, "physics_body_modified_death_probability", 0)
                ComponentSetValue2(expl_component, "explode_on_death_percent", 0)
            end

        end

        local enemy_data_new = ctx.entity_by_remote_id[remote_enemy_id]
        enemy_data_new.frame = frame
        local enemy_id = enemy_data_new.id

        for i, phys_component in ipairs(EntityGetComponent(enemy_id, "PhysicsBodyComponent") or {}) do
            local phys_info = phys_infos[i]
            if phys_component ~= nil and phys_component ~= 0 and phys_info ~= nil then
                deserialize_phys_component(phys_component, phys_info)
            end
        end
        for i, phys_component in ipairs(EntityGetComponent(enemy_id, "PhysicsBody2Component") or {}) do
            local phys_info = phys_infos_2[i]
            if phys_component ~= nil and phys_component ~= 0 and phys_info ~= nil then
                -- A physics body doesn't exist otherwise, causing a crash
                local initialized = ComponentGetValue2(phys_component, "mInitialized")
                if initialized then
                    deserialize_phys_component(phys_component, phys_info)
                end
            end
        end

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
        ::continue::
    end
end


function rpc.handle_enemy_health(enemy_health_data)
    -- GamePrint("Got enemy data: "..#enemy_data)
    for _, en_data in ipairs(enemy_health_data) do
        local remote_enemy_id = en_data.enemy_id
        local hp = en_data.hp
        local max_hp = en_data.max_hp

        if ctx.entity_by_remote_id[remote_enemy_id] == nil or not EntityGetIsAlive(ctx.entity_by_remote_id[remote_enemy_id].id) then
            goto continue
        end
        local enemy_data = ctx.entity_by_remote_id[remote_enemy_id]
        local enemy_id = enemy_data.id

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
        if projectileComponent ~= nil then
            local entity_that_shot    = ComponentGetValue2(projectileComponent, "mEntityThatShot")
            if entity_that_shot == 0 then
                rpc.replicate_projectile(np.SerializeEntity(projectile_id), position_x, position_y, target_x, target_y, shooter_id, initial_rng)
            end
        end
    end
end

rpc.opts_reliable()
function rpc.replicate_projectile(seri_ent, position_x, position_y, target_x, target_y, remote_source_ent, rng)
    if rng ~= nil then
        np.SetProjectileSpreadRNG(rng)
    end
    if ctx.entity_by_remote_id[remote_source_ent] == nil then
        return
    end
    local source_ent = ctx.entity_by_remote_id[remote_source_ent].id
    local ent = EntityCreateNew()
    np.DeserializeEntity(ent, seri_ent)
    GameShootProjectile(source_ent, position_x, position_y, target_x, target_y, ent)
end

return enemy_sync