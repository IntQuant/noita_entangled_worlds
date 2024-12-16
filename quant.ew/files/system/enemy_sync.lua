local item_sync = dofile_once("mods/quant.ew/files/system/item_sync.lua")
local effect_sync = dofile_once("mods/quant.ew/files/system/game_effect_sync/game_effect_sync.lua")
local stain_sync = dofile_once("mods/quant.ew/files/system/effect_data_sync/effect_data_sync.lua")
local EZWand = dofile_once("mods/quant.ew/files/lib/EZWand.lua")

local ffi = require("ffi")
local rpc = net.new_rpc_namespace()

local EnemyData = util.make_type({
    u32 = {"enemy_id"},
    f32 = {"x", "y", "vx", "vy"},
})

-- Variant of EnemyData for when we don't have any motion (or no VelocityComponent).
local EnemyDataNoMotion = util.make_type({
    u32 = {"enemy_id"},
    f32 = {"x", "y"},
})

local EnemyDataWorm = util.make_type({
    u32 = {"enemy_id"},
    f32 = {"x", "y", "vx", "vy", "tx", "ty"},
})

local EnemyDataKolmi = util.make_type({
    u32 = {"enemy_id"},
    f32 = {"x", "y", "vx", "vy"},
    bool = {"enabled"},
    vecfloat = {"legs"},
})

local EnemyDataMom = util.make_type({
    u32 = {"enemy_id"},
    f32 = {"x", "y", "vx", "vy"},
    vecbool = {"orbs"},
})

local EnemyDataFish = util.make_type({
    u32 = {"enemy_id"},
    f32 = {"x", "y", "vx", "vy"},
    u8 = {"r"},
})

local HpData = util.make_type({
    u32 = {"enemy_id"},
    f32 = {"hp", "max_hp"}
})

local should_wait = {}

local first = true

local FULL_TURN = math.pi * 2

local frame = 0

local enemy_sync = {}

local unsynced_enemys = {}

local dead_entities = {}
--this basically never happens, doesn't seem that useful anymore. Perhaps should be removed to conserve memory.
--local confirmed_kills = {}

local spawned_by_us = {}

-- HACK
local times_spawned_last_minute = {}

local DISTANCE_LIMIT = 128*6

for filename, _ in pairs(constants.phys_sync_allowed) do
    util.add_tag_to(filename, "prop_physics")
    -- Idk it just causes the minecart to not appear at all.
    -- util.replace_text_in(filename, 'kill_entity_after_initialized="1"', 'kill_entity_after_initialized="0"')
end

util.add_cross_call("ew_es_death_notify", function(enemy_id, responsible_id)
    local player_data = player_fns.get_player_data_by_local_entity_id(responsible_id)
    local responsible
    if player_data ~= nil then
        responsible = player_data.peer_id
    else
        responsible = responsible_id
    end
    local damage = EntityGetFirstComponentIncludingDisabled(enemy_id, "DamageModelComponent")
    table.insert(dead_entities, {enemy_id, responsible, ComponentGetValue2(damage, "wait_for_kill_flag_on_death")})
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

local function get_sync_entities(return_all)
    local entities = EntityGetWithTag("enemy") or {}
    table_extend(entities, EntityGetWithTag("ew_enemy_sync_extra"))
    table_extend(entities, EntityGetWithTag("plague_rat"))
    table_extend(entities, EntityGetWithTag("seed_f"))
    table_extend(entities, EntityGetWithTag("seed_e"))
    table_extend(entities, EntityGetWithTag("seed_d"))
    table_extend(entities, EntityGetWithTag("seed_c"))
    table_extend(entities, EntityGetWithTag("perk_fungus_tiny"))
    table_extend(entities, EntityGetWithTag("helpless_animal"))
    table_extend_filtered(entities, EntityGetWithTag("hittable"), function(ent)
        local name = EntityGetName(ent)
        local file = EntityGetFilename(ent)
        return name == "$item_essence_stone"
                or name == "$animal_fish_giga"
                or file == "data/entities/buildings/spittrap_left.xml"
                or file == "data/entities/buildings/spittrap_right.xml"
                or file == "data/entities/buildings/thundertrap_left.xml"
                or file == "data/entities/buildings/thundertrap_right.xml"
                or file == "data/entities/buildings/arrowtrap_left.xml"
                or file == "data/entities/buildings/arrowtrap_right.xml"
                or file == "data/entities/buildings/firetrap_left.xml"
                or file == "data/entities/buildings/firetrap_right.xml"
                          --data/entities/buildings/statue_trap_left.xml
                          --data/entities/buildings/statue_trap_right.xml
    end)
    table_extend_filtered(entities, EntityGetWithTag("prop_physics"), function (ent)
        local f = EntityGetFilename(ent)
        if f ~= nil then
            return constants.phys_sync_allowed[f]
        end
        return true
    end)

    local entities2 = {}
    if return_all then
        table_extend_filtered(entities2, entities, function(ent)
            return not EntityHasTag(ent, "ew_no_enemy_sync")
        end)
    else
        table_extend_filtered(entities2, entities, function(ent)
            local x, y = EntityGetTransform(ent)
            local has_anyone = EntityHasTag(ent, "worm")
                    or EntityGetFirstComponent(ent, "BossHealthBarComponent") ~= nil
                    or #EntityGetInRadiusWithTag(x, y, DISTANCE_LIMIT, "ew_peer") ~= 0
                    or #EntityGetInRadiusWithTag(x, y, DISTANCE_LIMIT, "polymorphed_player") ~= 0
            return has_anyone and not EntityHasTag(ent, "ew_no_enemy_sync")
        end)
    end

    return entities2
end

local was_held = {}

function enemy_sync.host_upload_entities()
    local entities = get_sync_entities()
    local enemy_data_list = {}
    for i, enemy_id in ipairs(entities) do
        if not world_exists_for(enemy_id) then
            goto continue
        end
        local filename = EntityGetFilename(enemy_id)
        filename = constants.interned_filename_to_index[filename] or filename

        local x, y, rot = EntityGetTransform(enemy_id)
        local character_data = EntityGetFirstComponentIncludingDisabled(enemy_id, "CharacterDataComponent")
        local vx, vy = 0, 0
        if character_data ~= nil then
            vx, vy = ComponentGetValue2(character_data, "mVelocity")
        else
            local velocity = EntityGetFirstComponentIncludingDisabled(enemy_id, "VelocityComponent")
            if velocity ~= nil then
                vx, vy = ComponentGetValue2(velocity, "mVelocity")
            end
        end
        local ai_component = EntityGetFirstComponentIncludingDisabled(enemy_id, "AnimalAIComponent")
        if ai_component ~= 0 and ai_component ~= nil then
            ComponentSetValue2(ai_component, "max_distance_to_cam_to_start_hunting", math.pow(2, 29))
        end

        local phys_info = util.get_phys_info(enemy_id, true)
        if phys_info == nil then
            goto continue
        end

        local hp, max_hp, has_hp = util.get_ent_health(enemy_id)
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

        local death_triggers = {}
        for _, com in ipairs(EntityGetComponent(enemy_id, "LuaComponent") or {}) do
            local script = ComponentGetValue2(com, "script_death")
            if script ~= nil and script ~= "" then
                table.insert(death_triggers, constants.interned_filename_to_index[script] or script)
            end
        end
        local en_data
        local worm = EntityGetFirstComponentIncludingDisabled(enemy_id, "WormAIComponent")
                or EntityGetFirstComponentIncludingDisabled(enemy_id, "BossDragonComponent")
        if EntityHasTag(enemy_id, "boss_centipede") then
            local legs = {}
            for _, leg in ipairs(EntityGetAllChildren(enemy_id, "foot")) do
                local limb = EntityGetFirstComponentIncludingDisabled(leg, "IKLimbComponent")
                local lx, ly = ComponentGetValue2(limb, "end_position")
                table.insert(legs, lx)
                table.insert(legs, ly)
            end
            en_data = EnemyDataKolmi {
                enemy_id = enemy_id,
                x = x,
                y = y,
                vx = vx,
                vy = vy,
                enabled = EntityGetFirstComponent(enemy_id, "BossHealthBarComponent", "disabled_at_start") ~= nil,
                legs = legs,
            }
        elseif EntityHasTag(enemy_id, "boss_wizard") then
            local orbs = {false, false, false, false, false, false, false, false}
            for _, child in ipairs(EntityGetAllChildren(enemy_id) or {}) do
                local var = EntityGetFirstComponentIncludingDisabled(child, "VariableStorageComponent")
                if EntityHasTag(child, "touchmagic_immunity") and var ~= nil then
                    local n = ComponentGetValue2(var, "value_int")
                    orbs[n] = true
                end
            end
            en_data = EnemyDataMom {
                enemy_id = enemy_id,
                x = x,
                y = y,
                vx = vx,
                vy = vy,
                orbs = orbs
            }
        elseif worm ~= nil then
            local tx, ty = ComponentGetValue2(worm, "mTargetVec")
            en_data = EnemyDataWorm {
                enemy_id = enemy_id,
                x = x,
                y = y,
                vx = vx,
                vy = vy,
                tx = tx,
                ty = ty,
            }
        elseif math.abs(vx) < 0.01 and math.abs(vy) < 0.01 then
            en_data = EnemyDataNoMotion {
                enemy_id = enemy_id,
                x = x,
                y = y,
            }
        elseif EntityGetFirstComponentIncludingDisabled(enemy_id, "AdvancedFishAIComponent") ~= nil then
            en_data = EnemyDataFish {
                enemy_id = enemy_id,
                x = x,
                y = y,
                vx = vx,
                vy = vy,
                r = math.floor((rot % FULL_TURN) / FULL_TURN * 255),
            }
        else
            en_data = EnemyData {
                enemy_id = enemy_id,
                x = x,
                y = y,
                vx = vx,
                vy = vy,
            }
        end

        local wand
        local inv = EntityGetFirstComponentIncludingDisabled(enemy_id, "Inventory2Component")
        if inv ~= nil then
            local item = ComponentGetValue2(inv, "mActualActiveItem")
            if item ~= nil and EntityGetIsAlive(item) then
                if not EntityHasTag(item, "ew_global_item") then
                    item_sync.make_item_global(item)
                else
                    wand = item_sync.get_global_item_id(item)
                    if wand == nil then
                        EntityRemoveTag(item, "ew_global_item")
                        goto continue
                    end
                    if not item_sync.is_my_item(wand) then
                        item_sync.take_authority(wand)
                    end
                    was_held[wand] = true
                end
            end
        end

        local effect_data = effect_sync.get_sync_data(enemy_id, true)

        local has_laser
        local animations = {}

        for _, sprite in ipairs(EntityGetComponent(enemy_id, "SpriteComponent") or {}) do
            local animation
            if sprite ~= nil then
                animation = ComponentGetValue2(sprite, "rect_animation")
            end
            table.insert(animations, animation)
            if ComponentHasTag(sprite, "laser_sight") then
                has_laser = true
            end
        end
        local laser
        if has_laser and EntityGetName(enemy_id) ~= "$animal_turret" then
            local ai = EntityGetFirstComponentIncludingDisabled(enemy_id, "AnimalAIComponent")
            if ai ~= nil then
                local target = ComponentGetValue2(ai, "mGreatestPrey")
                local peer = player_fns.get_player_data_by_local_entity_id(target)
                if peer ~= nil then
                    laser = peer.peer_id
                end
            end
        end

        local dont_cull = EntityGetFirstComponent(enemy_id, "BossHealthBarComponent") ~= nil
                or worm ~= nil
                or EntityHasTag(enemy_id, "seed_f")
                or EntityHasTag(enemy_id, "seed_e")
                or EntityHasTag(enemy_id, "seed_d")
                or EntityHasTag(enemy_id, "seed_c")
                or EntityGetFilename(enemy_id) == "data/entities/buildings/essence_eater.xml"

        local stains = stain_sync.get_stains(enemy_id)

        table.insert(enemy_data_list, {filename, en_data, phys_info, wand,
                                       effect_data, animations, dont_cull, death_triggers, stains, laser})
        ::continue::
    end

    rpc.handle_enemy_data(enemy_data_list, first)
    first = false
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
    for _, enemy_id in ipairs(entities) do
        if not EntityHasTag(enemy_id, "ew_replicated") then
            EntityKill(enemy_id)
        elseif not spawned_by_us[enemy_id] then
            EntityKill(enemy_id)
        end
    end
    for remote_id, enemy_data in pairs(ctx.entity_by_remote_id) do
        if frame > enemy_data.frame then
            EntityKill(enemy_data.id)
            ctx.entity_by_remote_id[remote_id] = nil
        end
    end
end

function enemy_sync.on_world_update_host()
    local rt = math.floor(tonumber(ModSettingGet("quant.ew.enemy_sync") or 2) + 0.5)
    local n = 0
    if rt == 3 then
        n = 2
    elseif rt == 2 then
        n = 1
    end
    if rt == 1 or GameGetFrameNum() % rt == n then
        enemy_sync.host_upload_entities()
    end
    if GameGetFrameNum() % 10 == 5 then
        host_upload_health()
    end
    for wand, _ in pairs(was_held) do
        if EntityGetRootEntity(wand) == wand then
            was_held[wand] = nil
            if item_sync.is_my_item(item_sync.get_global_item_id(wand)) then
                item_sync.make_item_global(wand)
            end
        end
    end
end

function enemy_sync.on_world_update_client()
    if GameGetFrameNum() % 12 == 1 then
        enemy_sync.client_cleanup()
    end
    if GameGetFrameNum() % (60*60) == 1 then
        times_spawned_last_minute = {}
    end
end

local kolmi_spawn

local function sync_enemy(enemy_info_raw, force_no_cull, host_fps)
    local filename = enemy_info_raw[1]
    filename = constants.interned_index_to_filename[filename] or filename

    local en_data = enemy_info_raw[2]
    local dont_cull = enemy_info_raw[7]
    local death_triggers = enemy_info_raw[8]
    local stains = enemy_info_raw[9]
    local has_laser = enemy_info_raw[10]
    local remote_enemy_id = en_data.enemy_id
    local x, y = en_data.x, en_data.y
    if not force_no_cull and not dont_cull  then
        local my_x, my_y = EntityGetTransform(ctx.my_player.entity)
        if my_x == nil then
            goto continue
        end
        local c_x, c_y = GameGetCameraPos()
        local dx, dy = my_x - x, my_y - y
        local cdx, cdy = c_x - x, c_y - y
        if dx * dx + dy * dy > DISTANCE_LIMIT * DISTANCE_LIMIT and cdx * cdx + cdy * cdy > DISTANCE_LIMIT * DISTANCE_LIMIT then
            if ctx.entity_by_remote_id[remote_enemy_id] ~= nil then
                EntityKill(ctx.entity_by_remote_id[remote_enemy_id].id)
                ctx.entity_by_remote_id[remote_enemy_id] = nil
            end
            unsynced_enemys[remote_enemy_id] = enemy_info_raw
            goto continue
        else
            unsynced_enemys[remote_enemy_id] = nil
        end
    else
        unsynced_enemys[remote_enemy_id] = nil
    end
    local vx = 0
    local vy = 0
    if ffi.typeof(en_data) ~= EnemyDataNoMotion then
        vx, vy = en_data.vx, en_data.vy
    end
    local phys_infos = enemy_info_raw[3]
    local gid = enemy_info_raw[4]
    local effects = enemy_info_raw[5]
    local animation = enemy_info_raw[6]
    local has_died = filename == nil

    local frame_now = GameGetFrameNum()

    --[[if confirmed_kills[remote_enemy_id] then
        goto continue
    end]]

    if ctx.entity_by_remote_id[remote_enemy_id] ~= nil and not EntityGetIsAlive(ctx.entity_by_remote_id[remote_enemy_id].id) then
        ctx.entity_by_remote_id[remote_enemy_id] = nil
    end

    if ctx.entity_by_remote_id[remote_enemy_id] == nil then
        if filename == nil or filename == "" or not ModDoesFileExist(filename) then
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
        enemy_id = EntityLoad(filename, x, y)
        spawned_by_us[enemy_id] = true
        EntityAddTag(enemy_id, "ew_replicated")
        EntityAddTag(enemy_id, "polymorphable_NOT")
        for _, com in ipairs(EntityGetComponent(enemy_id, "LuaComponent") or {}) do
            local script = ComponentGetValue2(com, "script_damage_received")
            if (script ~= nil
                    and (script == "data/scripts/animals/leader_damage.lua"
                    or script == "data/scripts/animals/giantshooter_death.lua"
                    or script == "data/scripts/animals/blob_damage.lua"))
                    or ComponentGetValue2(com, "script_source_file") == "data/scripts/props/suspended_container_physics_objects.lua" then
                EntityRemoveComponent(enemy_id, com)
            end
        end
        EntityAddComponent2(enemy_id, "LuaComponent", {_tags="ew_immortal", script_damage_about_to_be_received = "mods/quant.ew/files/resource/cbs/immortal.lua"})
        local damage_component = EntityGetFirstComponentIncludingDisabled(enemy_id, "DamageModelComponent")
        if damage_component and damage_component ~= 0 then
            ComponentSetValue2(damage_component, "wait_for_kill_flag_on_death", true)
        end
        for _, name in ipairs({"AnimalAIComponent", "PhysicsAIComponent", "CameraBoundComponent", "AdvancedFishAIComponent", "AIAttackComponent"}) do
            local ai_component = EntityGetFirstComponentIncludingDisabled(enemy_id, name)
            if ai_component ~= 0 then
                EntityRemoveComponent(enemy_id, ai_component)
            end
        end
        ctx.entity_by_remote_id[remote_enemy_id] = {id = enemy_id, frame = frame_now}

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
        local pick_up = EntityGetFirstComponentIncludingDisabled(enemy_id, "ItemPickUpperComponent")
        if pick_up ~= nil then
            EntityRemoveComponent(enemy_id, pick_up)
        end
        for _, sprite in pairs(EntityGetComponent(enemy_id, "SpriteComponent", "character") or {}) do
            ComponentRemoveTag(sprite, "character")
        end

        local ghost = EntityGetFirstComponentIncludingDisabled(enemy_id, "GhostComponent")
        if ghost ~= nil then
            ComponentSetValue2(ghost, "die_if_no_home", false)
        end
        if not EntityHasTag(enemy_id, "effectable_prop") then
            util.make_ephemerial(enemy_id)
        end
    end

    local enemy_data_new = ctx.entity_by_remote_id[remote_enemy_id]
    enemy_data_new.frame = frame_now
    local enemy_id = enemy_data_new.id

    if not has_died then
        local laser = EntityGetFirstComponentIncludingDisabled(enemy_id, "LaserEmitterComponent", "ew_laser")
        if has_laser then
            if laser == nil then
                laser = EntityAddComponent2(enemy_id, "LaserEmitterComponent", {_tags = "ew_laser"})
                ComponentObjectSetValue2(laser, "laser", "max_cell_durability_to_destroy", 0)
                ComponentObjectSetValue2(laser, "laser", "damage_to_cells", 0)
                ComponentObjectSetValue2(laser, "laser", "max_length", 1024)
                ComponentObjectSetValue2(laser, "laser", "beam_radius", 0)
                ComponentObjectSetValue2(laser, "laser", "beam_particle_chance", 80)
                ComponentObjectSetValue2(laser, "laser", "beam_particle_fade", 0)
                ComponentObjectSetValue2(laser, "laser", "hit_particle_chance", 0)
                ComponentObjectSetValue2(laser, "laser", "audio_enabled", false)
                ComponentObjectSetValue2(laser, "laser", "damage_to_entities", 0)
                ComponentObjectSetValue2(laser, "laser", "beam_particle_type", 225)
            end
            local target = ctx.players[has_laser].entity
            local lx, ly = EntityGetTransform(target)
            if lx ~= nil then
                local did_hit, _, _ = RaytracePlatforms(x, y, lx, ly)
                ComponentSetValue2(laser, "is_emitting", not did_hit)
                if not did_hit then
                    local dy = ly - y
                    local dx = lx - x
                    local theta = math.atan2(dy, dx)
                    ComponentSetValue2(laser, "laser_angle_add_rad", theta)
                    ComponentObjectSetValue2(laser, "laser", "max_length", math.sqrt(dx * dx + dy * dy))
                end
            end
        elseif laser ~= nil then
            ComponentSetValue2(laser, "is_emitting", false)
        end
        if not util.set_phys_info(enemy_id, phys_infos, host_fps) or enemy_id == kolmi_spawn then
            local m = host_fps / ctx.my_player.fps
            vx, vy = vx * m, vy * m
            local character_data = EntityGetFirstComponentIncludingDisabled(enemy_id, "CharacterDataComponent")
            if character_data ~= nil then
                ComponentSetValue2(character_data, "mVelocity", vx, vy)
            else
                local velocity_data = EntityGetFirstComponentIncludingDisabled(enemy_id, "VelocityComponent")
                if velocity_data ~= nil then
                    ComponentSetValue2(velocity_data, "mVelocity", vx, vy)
                end
            end
            if ffi.typeof(en_data) == EnemyDataFish then
                EntitySetTransform(enemy_id, x, y, en_data.r / 255 * FULL_TURN)
            else
                EntitySetTransform(enemy_id, x, y)
            end
        end
        local worm = EntityGetFirstComponentIncludingDisabled(enemy_id, "WormAIComponent")
            or EntityGetFirstComponentIncludingDisabled(enemy_id, "BossDragonComponent")
        if worm ~= nil and ffi.typeof(en_data) == EnemyDataWorm then
            local tx, ty = en_data.tx, en_data.ty
            ComponentSetValue2(worm, "mTargetVec", tx, ty)
        end
        if ffi.typeof(en_data) == EnemyDataKolmi and en_data.enabled then
            if kolmi_spawn ~= enemy_id then
                for _, c in ipairs(EntityGetComponentIncludingDisabled(enemy_id, "LuaComponent") or {}) do
                    EntityRemoveComponent(enemy_id, c)
                end
                kolmi_spawn = enemy_id
            end
            EntitySetComponentsWithTagEnabled(enemy_id, "enabled_at_start", false)
            EntitySetComponentsWithTagEnabled(enemy_id, "disabled_at_start", true)
            for i, leg in ipairs(EntityGetAllChildren(enemy_id, "foot")) do
                local limb = EntityGetFirstComponentIncludingDisabled(leg, "IKLimbComponent")
                ComponentSetValue2(limb, "end_position", en_data.legs[2 * i - 2], en_data.legs[2 * i - 1])
            end
        end

        local indexed = {}
        for _, com in ipairs(EntityGetComponent(enemy_id, "LuaComponent") or {}) do
            local script = ComponentGetValue2(com, "script_death")
            local has = false
            for _, inx in ipairs(death_triggers) do
                local script2 = constants.interned_index_to_filename[inx] or inx
                if script == script2 then
                    has = true
                    indexed[script] = true
                end
            end
            if not has then
                ComponentSetValue2(com, "script_death", "")
            end
        end
        for _, inx in ipairs(death_triggers) do
            local script = constants.interned_index_to_filename[inx] or inx
            if indexed[script] == nil then
                EntityAddComponent(enemy_id, "LuaComponent", { script_death = script,
                            execute_every_n_frame = "-1"})
            end
        end
        if ffi.typeof(en_data) == EnemyDataMom then
            local orbs = en_data.orbs
            for _, child in ipairs(EntityGetAllChildren(enemy_id) or {}) do
                local var = EntityGetFirstComponentIncludingDisabled(child, "VariableStorageComponent")
                local damage_component = EntityGetFirstComponentIncludingDisabled(child, "DamageModelComponent")
                if EntityHasTag(child, "touchmagic_immunity") and var ~= nil then
                    local n = ComponentGetValue2(var, "value_int")
                    if orbs[n - 1] then
                        ComponentSetValue2(damage_component, "wait_for_kill_flag_on_death", true)
                    else
                        ComponentSetValue2(damage_component, "wait_for_kill_flag_on_death", false)
                        EntityKill(child)
                    end
                end
            end
        end
        effect_sync.apply_effects(effects, enemy_id, true)
        if stains ~= nil then
            stain_sync.sync_stains(stains, enemy_id)
        end
    end

    local inv = EntityGetFirstComponentIncludingDisabled(enemy_id, "Inventory2Component")
    local item
    if inv ~= nil then
        item = ComponentGetValue2(inv, "mActualActiveItem")
    end
    if gid ~= nil and (item == nil or item == 0 or not EntityGetIsAlive(item)) then
        local wand = item_sync.find_by_gid(gid)
        if wand ~= nil then
            EntityAddTag(wand, "ew_client_item")
            local ezwand = EZWand(wand)
            ezwand:GiveTo(enemy_id)
            np.SetActiveHeldEntity(enemy_id, wand, false, false)
        else
            if should_wait[gid] == nil or should_wait[gid] < GameGetFrameNum() then
                item_sync.rpc.request_send_again(gid)
                should_wait[gid] = GameGetFrameNum() + 15
            end
        end
    end

    for i, sprite in pairs(EntityGetComponent(enemy_id, "SpriteComponent") or {}) do
        if animation[i] ~= nil then
            ComponentSetValue2(sprite, "rect_animation", animation[i])
            ComponentSetValue2(sprite, "next_rect_animation", animation[i])
        end
    end

    ::continue::
end

rpc.opts_reliable()
function rpc.handle_death_data(death_data)
    for _, remote_data in ipairs(death_data) do
        local remote_id = remote_data[1]
        --[[if confirmed_kills[remote_id] then
            GamePrint("Remote id has been killed already..?")
            goto continue
        end
        confirmed_kills[remote_id] = true]]
        local responsible_entity = 0
        local peer_data = player_fns.peer_get_player_data(remote_data[2], true)
        if peer_data ~= nil then
            responsible_entity = peer_data.entity
        elseif ctx.entity_by_remote_id[remote_data[2]] ~= nil then
            responsible_entity = ctx.entity_by_remote_id[remote_data[2]]
        end

        if unsynced_enemys[remote_id] ~= nil then
            sync_enemy(unsynced_enemys[remote_id], true, 60)
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
            if not remote_data[3] then
                EntityKill(enemy_id)
            else
                ComponentSetValue2(damage_component, "wait_for_kill_flag_on_death", true)
                ComponentSetValue2(damage_component, "kill_now", true)
            end
            ctx.entity_by_remote_id[remote_id] = nil
        end
        ::continue::
    end
end

function rpc.handle_enemy_data(enemy_data, is_first)
    if is_first then
        for _, n in pairs(ctx.entity_by_remote_id) do
            EntityKill(n.id)
        end
        ctx.entity_by_remote_id = {}
    end
    frame = GameGetFrameNum()
    for _, enemy_info_raw in ipairs(enemy_data) do
        sync_enemy(enemy_info_raw, false, ctx.rpc_player_data.fps)
    end
end

function rpc.handle_enemy_health(enemy_health_data)
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
            if EntityGetName(enemy_id) ~= "$animal_boss_sky" then
                util.set_ent_health(enemy_id, {dmg*2, dmg*2})
            else
                util.set_ent_health(enemy_id, {hp + dmg, max_hp})
            end
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
                rpc.replicate_projectile(util.serialize_entity(projectile_id), position_x, position_y, target_x, target_y, shooter_id, initial_rng)
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
    local ent = util.deserialize_entity(seri_ent)
    GameShootProjectile(source_ent, position_x, position_y, target_x, target_y, ent)
end

return enemy_sync