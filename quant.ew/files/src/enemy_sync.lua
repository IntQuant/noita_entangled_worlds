local util = dofile_once("mods/quant.ew/files/src/util.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")

local enemy_sync = {}

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
    local entities = EntityGetWithTag("enemy") -- TODO maybe only sync those close to players?
    table_extend_filtered(entities, EntityGetWithTag("projectile"), function (ent)
        return not (EntityHasTag(ent, "ew_shot_by_player") or EntityHasTag(ent, "projectile_player"))
    end)
    return entities
end

local previous_sync_entities = {}

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
        if ai_component ~= nil then
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

    local dead_entities = {}

    local i = 1
    while GlobalsGetValue("ew_enemy_death_"..i, "0") ~= "0" do
        local enemy_id = tonumber(GlobalsGetValue("ew_enemy_death_"..i, "0"))
        GlobalsSetValue("ew_enemy_death_"..i, "0")
        -- GamePrint("Entity is no longer alive: "..enemy_id)
        table.insert(dead_entities, enemy_id)
        i = i + 1
    end

    previous_sync_entities = entities
    return enemy_data_list, dead_entities
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
            print("Despawning stale "..remote_id)
            EntityKill(enemy_data.id)
            ctx.entity_by_remote_id[remote_id] = nil
        end
    end
end

function enemy_sync.handle_death_data(death_data)
    for _, remote_id in ipairs(death_data) do
        local enemy_data = ctx.entity_by_remote_id[remote_id]
        if enemy_data ~= nil then
            local enemy_id = enemy_data.id

            local current_hp = util.get_ent_health(enemy_id)
            local dmg = current_hp
            if dmg > 0 then
                EntityInflictDamage(enemy_id, dmg+0.1, "DAMAGE_CURSE", "", "NONE", 0, 0, GameGetWorldStateEntity())
            end
            EntityInflictDamage(enemy_id, 1000000000, "DAMAGE_CURSE", "", "NONE", 0, 0, GameGetWorldStateEntity()) -- Just to be sure
            util.set_ent_health(enemy_id, {0, 0})
        end
    end
end

function enemy_sync.handle_enemy_data(enemy_data)
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
            EntityAddComponent2(enemy_id, "LuaComponent", {script_damage_about_to_be_received = "mods/quant.ew/files/cbs/immortal.lua"})
            local ai_component = EntityGetFirstComponentIncludingDisabled(enemy_id, "AnimalAIComponent")
            if ai_component ~= nil then
                EntityRemoveComponent(enemy_id, ai_component)
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
            -- GamePrint("Dealing dmg "..dmg)
            EntityInflictDamage(enemy_id, dmg, "DAMAGE_CURSE", "", "NONE", 0, 0, GameGetWorldStateEntity())
        end
        util.set_ent_health(enemy_id, {hp, max_hp})
        ::continue::
    end
end

return enemy_sync