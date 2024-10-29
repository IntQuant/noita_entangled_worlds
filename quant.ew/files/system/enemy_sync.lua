local util = dofile_once("mods/quant.ew/files/core/util.lua")
local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/core/player_fns.lua")
local inventory_helper = dofile_once("mods/quant.ew/files/core/inventory_helper.lua")
local effect_sync = dofile_once("mods/quant.ew/files/system/game_effect_sync/game_effect_sync.lua")
local np = require("noitapatcher")

local ffi = require("ffi")
local rpc = net.new_rpc_namespace()

local EnemyData = util.make_type({
    u32 = {"enemy_id"},
    f32 = {"x", "y", "vx", "vy"},
    bool = {"drop_gold"},
})

-- Variant of EnemyData for when we don't have any motion (or no VelocityComponent).
local EnemyDataNoMotion = util.make_type({
    u32 = {"enemy_id"},
    f32 = {"x", "y"},
    bool = {"drop_gold"}
})

local EnemyDataWorm = util.make_type({
    u32 = {"enemy_id"},
    f32 = {"x", "y", "vx", "vy", "tx", "ty"},
    bool = {"drop_gold"}
})

local EnemyDataKolmi = util.make_type({
    u32 = {"enemy_id"},
    f32 = {"x", "y", "vx", "vy"},
    bool = {"enabled", "drop_gold"},
})

local EnemyDataFish = util.make_type({
    u32 = {"enemy_id"},
    f32 = {"x", "y", "vx", "vy"},
    u8 = {"r"},
    bool = {"drop_gold"},
})

--local EnemyDataSniper = util.make_type({
--    u32 = {"enemy_id"},
--    f32 = {"x", "y", "vx", "vy"},
--    bool = {"aiming"},
--})

local HpData = util.make_type({
    u32 = {"enemy_id"},
    f32 = {"hp", "max_hp"}
})

--local HpDataMom = util.make_type({
--    u32 = {"enemy_id"},
--    f32 = {"hp", "max_hp", "hp1", "hp2", "hp3", "hp4"}
--})

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

local wands = {}

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

local function kill(entity)
    local parent = EntityGetParent(entity)
    if parent ~= nil then
        EntityKill(parent)
    end
    EntityKill(entity)
end

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
    px, py = PhysicsPosToGamePos(px, py)
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
    local x, y = GamePosToPhysicsPos(phys_info.x, phys_info.y)
    if ffi.typeof(phys_info) == PhysDataNoMotion then
        np.PhysBodySetTransform(phys_component, x, y, phys_info.r / 255 * FULL_TURN, 0, 0, 0)
    else
        np.PhysBodySetTransform(phys_component, x, y, phys_info.r / 255 * FULL_TURN, phys_info.vx, phys_info.vy, phys_info.vr)
    end
end

local function get_sync_entities(return_all)
    local entities = {}
    table_extend_filtered(entities, EntityGetWithTag("enemy"), function (ent)
        return not EntityHasTag(ent, "ew_no_enemy_sync") and not EntityHasTag(ent, "wand_ghost")
    end)
    table_extend(entities, EntityGetWithTag("ew_enemy_sync_extra"))
    table_extend(entities, EntityGetWithTag("mimic_potion"))
    table_extend(entities, EntityGetWithTag("plague_rat"))
    table_extend(entities, EntityGetWithTag("perk_fungus_tiny"))
    table_extend(entities, EntityGetWithTag("helpless_animal"))
    table_extend_filtered(entities, EntityGetWithTag("prop_physics"), function (ent)
        return constants.phys_sync_allowed[EntityGetFilename(ent)]
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
            return has_anyone and not EntityHasTag(ent, "ew_no_enemy_sync")
        end)
    end

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

        local x, y, rot = EntityGetTransform(enemy_id)
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
                local ret, info = pcall(serialize_phys_component, phys_component)
                if not ret then
                    GamePrint("Physics component has no body, deleting entity")
                    kill(enemy_id)
                    goto continue
                end
                table.insert(phys_info, info)
            end
        end

        for _, phys_component in ipairs(EntityGetComponent(enemy_id, "PhysicsBody2Component") or {}) do
            if phys_component ~= nil and phys_component ~= 0 then
                not_ephemerial = true
                local initialized = ComponentGetValue2(phys_component, "mInitialized")
                if initialized then
                    local ret, info = pcall(serialize_phys_component, phys_component)
                    if not ret then
                        GamePrint("Physics component has no body, deleting entity")
                        kill(enemy_id)
                        goto continue
                    end
                    table.insert(phys_info_2, info)
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

        local drop_gold = false
        for _, com in ipairs(EntityGetComponent(enemy_id, "LuaComponent") or {}) do
            if ComponentGetValue2(com, "script_death") == "data/scripts/items/drop_money.lua" then
                drop_gold = true
            end
        end
        local en_data
        local worm = EntityGetFirstComponentIncludingDisabled(enemy_id, "WormAIComponent")
        if EntityHasTag(enemy_id, "boss_centipede") then
            en_data = EnemyDataKolmi {
                enemy_id = enemy_id,
                x = x,
                y = y,
                vx = vx,
                vy = vy,
                enabled = EntityGetFirstComponent(enemy_id, "BossHealthBarComponent", "disabled_at_start") ~= nil,
                drop_gold = drop_gold
            }
        elseif worm ~= nil then
            local tx, ty = ComponentGetValue2(worm, "mRandomTarget")
            en_data = EnemyDataWorm {
                enemy_id = enemy_id,
                x = x,
                y = y,
                vx = vx,
                vy = vy,
                tx = tx,
                ty = ty,
                drop_gold = drop_gold
            }
        elseif math.abs(vx) < 0.01 and math.abs(vy) < 0.01 then
            en_data = EnemyDataNoMotion {
                enemy_id = enemy_id,
                x = x,
                y = y,
                drop_gold = drop_gold
            }
        elseif EntityGetFirstComponentIncludingDisabled(enemy_id, "AdvancedFishAIComponent") ~= nil then
            en_data = EnemyDataFish {
                enemy_id = enemy_id,
                x = x,
                y = y,
                vx = vx,
                vy = vy,
                r = math.floor((rot % FULL_TURN) / FULL_TURN * 255),
                drop_gold = drop_gold
            }
        else
            en_data = EnemyData {
                enemy_id = enemy_id,
                x = x,
                y = y,
                vx = vx,
                vy = vy,
                drop_gold = drop_gold
            }
        end

        local has_wand = false
        local inv = EntityGetFirstComponentIncludingDisabled(enemy_id, "Inventory2Component")
        if inv ~= nil then
            local item = ComponentGetValue2(inv, "mActualActiveItem")
            if item ~= nil and EntityGetIsAlive(item) then
                if wands[enemy_id] == nil then
                    wands[enemy_id] = inventory_helper.serialize_single_item(item)
                end
                has_wand = true
            end
        end
        if not has_wand and wands[enemy_id] ~= nil then
            table.remove(wands, enemy_id)
        end

        local effect_data = effect_sync.get_sync_data(enemy_id, true)

        local sprite = EntityGetFirstComponent(enemy_id, "SpriteComponent")
        local animation
        if sprite ~= nil then
            animation = ComponentGetValue2(sprite, "rect_animation")
        end

        local dont_cull = EntityHasTag(enemy_id, "worm") or EntityGetFirstComponent(enemy_id, "BossHealthBarComponent") ~= nil

        table.insert(enemy_data_list, {filename, en_data, not_ephemerial, phys_info, phys_info_2, has_wand, effect_data, animation, dont_cull})
        ::continue::
    end

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
    for _, enemy_id in ipairs(entities) do
        if not EntityHasTag(enemy_id, "ew_replicated") then
            --local filename = EntityGetFilename(enemy_id)
            --print("Despawning unreplicated "..enemy_id.." "..filename)
            kill(enemy_id)
        elseif not spawned_by_us[enemy_id] then
            local filename = EntityGetFilename(enemy_id)
            print("Despawning persisted "..enemy_id.." "..filename)
            kill(enemy_id)
        else
            local cull = EntityGetFirstComponentIncludingDisabled(enemy_id, "VariableStorageComponent", "ew_cull")
            if cull ~= nil and ComponentGetValue2(cull, "value_int") + 120 < GameGetFrameNum() then
                kill(enemy_id)
            end
        end
    end
    local frame = GameGetFrameNum()
    for remote_id, enemy_data in pairs(ctx.entity_by_remote_id) do
        if frame - enemy_data.frame > 60*2 then
            --print("Despawning stale "..remote_id.." "..enemy_data.id)
            kill(enemy_data.id)
            ctx.entity_by_remote_id[remote_id] = nil
        end
    end
    for _, ent in ipairs(EntityGetWithTag("ew_synced_entity") or {}) do
        if #(EntityGetAllChildren(ent) or {}) == 0 then
            EntityKill(ent)
        end
    end
end

function enemy_sync.on_world_update_host()
    local int = 3 --ctx.proxy_opt.enemy_sync_interval
    local num = 2
    if int == 1 then
        num = 0
    elseif int == 2 then
        num = 1
    end
    if GameGetFrameNum() % int == num then
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

local function sync_enemy(enemy_info_raw, force_no_cull)
    local filename = enemy_info_raw[1]
    filename = constants.interned_index_to_filename[filename] or filename

    local en_data = enemy_info_raw[2]
    local dont_cull = enemy_info_raw[9]
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
                kill(ctx.entity_by_remote_id[remote_enemy_id])
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
    local not_ephemerial = enemy_info_raw[3]
    local phys_infos = enemy_info_raw[4]
    local phys_infos_2 = enemy_info_raw[5]
    local has_wand = enemy_info_raw[6]
    local effects = enemy_info_raw[7]
    local animation = enemy_info_raw[8]
    local has_died = filename == nil

    local frame = GameGetFrameNum()

    --[[if confirmed_kills[remote_enemy_id] then
        goto continue
    end]]

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
        for _, com in ipairs(EntityGetComponent(enemy_id, "LuaComponent") or {}) do
            local script = ComponentGetValue2(com, "script_damage_received")
            if script ~= nil and (script == "data/scripts/animals/leader_damage.lua" or script == "data/scripts/animals/giantshooter_death.lua" or script == "data/scripts/animals/blob_damage.lua") then
                EntityRemoveComponent(enemy_id, com)
            end
        end
        if not dont_cull then
            EntityAddComponent2(enemy_id, "VariableStorageComponent", {_tags="ew_cull", value_int = GameGetFrameNum()})
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
        local pick_up = EntityGetFirstComponentIncludingDisabled(enemy_id, "ItemPickUpperComponent")
        if pick_up ~= nil then
            EntityRemoveComponent(enemy_id, pick_up)
        end
        for _, sprite in pairs(EntityGetComponent(enemy_id, "SpriteComponent", "character") or {}) do
            ComponentAddTag(sprite, "ew_sprite")
            ComponentRemoveTag(sprite, "character")
        end
    end

    local enemy_data_new = ctx.entity_by_remote_id[remote_enemy_id]
    enemy_data_new.frame = frame
    local enemy_id = enemy_data_new.id

    if not dont_cull then
        ComponentSetValue2(EntityGetFirstComponentIncludingDisabled(enemy_id, "VariableStorageComponent", "ew_cull"), "value_int", GameGetFrameNum())
    end

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
        if ffi.typeof(en_data) == EnemyDataFish then
            EntitySetTransform(enemy_id, x, y, en_data.r / 255 * FULL_TURN)
        else
            EntitySetTransform(enemy_id, x, y)
        end
        local worm = EntityGetFirstComponentIncludingDisabled(enemy_id, "WormAIComponent")
        if worm ~= nil and ffi.typeof(en_data) == EnemyDataWorm then
            local tx, ty = en_data.tx, en_data.ty
            ComponentSetValue2(worm, "mRandomTarget", tx, ty)
        end
        if ffi.typeof(en_data) == EnemyDataKolmi and en_data.enabled then
            local lua_components = EntityGetComponentIncludingDisabled(enemy_id, "LuaComponent") or {}
            for _, c in ipairs(lua_components) do
                EntityRemoveComponent(enemy_id, c)
            end
            EntitySetComponentsWithTagEnabled(enemy_id, "enabled_at_start", false)
            EntitySetComponentsWithTagEnabled(enemy_id, "disabled_at_start", true)
        end
    end
    if not en_data.drop_gold then
        for _, com in ipairs(EntityGetComponent(enemy_id, "LuaComponent") or {}) do
            if ComponentGetValue2(com, "script_death") == "data/scripts/items/drop_money.lua" then
                ComponentSetValue2(com, "script_death", "")
            end
        end
    end

    local inv = EntityGetFirstComponentIncludingDisabled(enemy_id, "Inventory2Component")
    local item
    if inv ~= nil then
        item = ComponentGetValue2(inv, "mActualActiveItem")
    end
    if has_wand and item == nil then
        if wands[remote_enemy_id] ~= nil then
            local wand = inventory_helper.deserialize_single_item(wands[remote_enemy_id])
            EntityAddTag(wand, "ew_client_item")
            local found = false
            for _, child in ipairs(EntityGetAllChildren(enemy_id) or {}) do
                if EntityGetName(child) == "inventory_quick" then
                    EntityAddChild(child, wand)
                    found = true
                    break
                end
            end
            if not found then
                local inv_quick = EntityCreateNew("inventory_quick")
                EntityAddChild(enemy_id, inv_quick)
                EntityAddChild(inv_quick, wand)
                EntityAddComponent2(enemy_id, "Inventory2Component")
            end
            EntitySetComponentsWithTagEnabled(wand, "enabled_in_world", false)
            EntitySetComponentsWithTagEnabled(wand, "enabled_in_hand", true)
            EntitySetComponentsWithTagEnabled(wand, "enabled_in_inventory", false)
            np.SetActiveHeldEntity(enemy_id, wand, false, false)
        else
            rpc.request_wand(ctx.my_id, remote_enemy_id)
        end
    end
    if not has_wand and wands[remote_enemy_id] ~= nil then
        table.remove(wands, remote_enemy_id)
    end

    effect_sync.apply_effects(effects, enemy_id, true)

    for _, sprite in pairs(EntityGetComponent(enemy_id, "SpriteComponent", "ew_sprite") or {}) do
        ComponentSetValue2(sprite, "rect_animation", animation)
        ComponentSetValue2(sprite, "next_rect_animation", animation)
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
            sync_enemy(unsynced_enemys[remote_id], true)
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
            kill(enemy_id)
        end
        if wands[remote_id] ~= nil then
            table.remove(wands, remote_id)
        end
        ::continue::
    end
end

function rpc.send_wand(peer_id, remote_enemy_id, wand)
    if ctx.my_id == peer_id and wand ~= nil then
        wands[remote_enemy_id] = wand
    end
end

function rpc.request_wand(peer_id, remote_enemy_id)
    if ctx.my_id == ctx.host_id then
        rpc.send_wand(peer_id, remote_enemy_id, wands[remote_enemy_id])
    end
end

function rpc.handle_enemy_data(enemy_data)
    for _, enemy_info_raw in ipairs(enemy_data) do
        sync_enemy(enemy_info_raw, false)
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