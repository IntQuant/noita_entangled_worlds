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

function enemy_sync.host_upload_entities()
    local entities = EntityGetWithTag("enemy") -- TODO maybe only sync those close to players?
    table_extend_filtered(entities, EntityGetWithTag("projectile"), function (ent)
        return not EntityHasTag(ent, "projectile_player")
    end)
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
        local hp, max_hp = util.get_ent_health(enemy_id)
        table.insert(enemy_data_list, {enemy_id, filename, x, y, vx, vy, hp, max_hp})
        ::continue::
    end
    --print(#enemy_data_list)
    return enemy_data_list
end

function enemy_sync.client_cleanup()
    local enemy_list = EntityGetWithTag("enemy")
    for i, enemy_id in ipairs(enemy_list) do
        if not EntityHasTag(enemy_id, "ew_replicated") then
            print("Despawning unreplicated "..enemy_id)
            EntityKill(enemy_id)
        end
    end
    local frame = GameGetFrameNum()
    for remote_id, enemy_data in pairs(ctx.enemy_by_remote_id) do
        if frame - enemy_data.frame > 60*10 then
            print("Despawning stale "..remote_id)
            EntityKill(enemy_data.id)
            ctx.enemy_by_remote_id[remote_id] = nil
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

        local frame = GameGetFrameNum()
        
        if ctx.enemy_by_remote_id[remote_enemy_id] ~= nil and not EntityGetIsAlive(ctx.enemy_by_remote_id[remote_enemy_id].id) then
            ctx.enemy_by_remote_id[remote_enemy_id] = nil
        end
        
        if ctx.enemy_by_remote_id[remote_enemy_id] == nil then
            local enemy_id = EntityLoad(filename, x, y)
            EntityAddTag(enemy_id, "ew_replicated")
            local ai_component = EntityGetFirstComponentIncludingDisabled(enemy_id, "AnimalAIComponent")
            if ai_component ~= 0 then                
                EntityRemoveComponent(enemy_id, ai_component)
            end
            ctx.enemy_by_remote_id[remote_enemy_id] = {id = enemy_id, frame = frame}
        end

        local enemy_data = ctx.enemy_by_remote_id[remote_enemy_id]
        enemy_data.frame = frame
        local enemy_id = enemy_data.id

        local character_data = EntityGetFirstComponentIncludingDisabled(enemy_id, "CharacterDataComponent")
        if character_data ~= 0 then
            ComponentSetValue2(character_data, "mVelocity", vx, vy)
        end
        local character_data = EntityGetFirstComponentIncludingDisabled(enemy_id, "VelocityComponent")
        if character_data ~= 0 then
            ComponentSetValue2(character_data, "mVelocity", vx, vy)
        end

        local px, py = EntityGetTransform(enemy_id)
        local tp_limit = 30
        local alpha = 0.2
        -- if math.pow(px-x, 2) + math.pow(py-y, 2) > math.pow(tp_limit, 2) then
            EntitySetTransform(enemy_id, x, y)
        -- else
            -- EntitySetTransform(util.lerp(px, x, alpha), util.lerp(py, y, alpha))
        -- end
        util.set_ent_health(enemy_id, {hp, max_hp})
    end
end

return enemy_sync