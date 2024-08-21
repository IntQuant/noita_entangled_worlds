local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local player_fns = dofile_once("mods/quant.ew/files/core/player_fns.lua")

local tether_length = ctx.proxy_opt.tether_length
local tether_length_2 = tether_length + 200

local module = {}

local function is_in_box(x1, x2, y1, y2, x, y)
    return x1 < x and x < x2 and y1 < y and y < y2
end

local function in_normal_hm(list, x, y)
    local x1 = -1024
    local x2 = 512
    local dy = 338
    for _, y1 in ipairs(list) do
        if is_in_box(x1, x2, y1, y1 + dy, x, y) then
            return true
        end
    end
    return false
end

local function not_in_hm(x, y)
    local list = {1198, 2734, 4782, 6318, 8366, 10414}
    return not (in_normal_hm(list, x, y) or is_in_box(1536, 2726, 12798, 13312, x, y))
end

function module.on_client_spawned(peer_id, new_playerdata)
    local is_host = peer_id == ctx.host_id
    if is_host then
        local zone_ent = EntityLoad("mods/quant.ew/files/system/player_tether/zone_entity.xml")
        EntityAddChild(new_playerdata.entity, zone_ent)
        local particle_component = EntityGetFirstComponentIncludingDisabled(zone_ent, "ParticleEmitterComponent")
        ComponentSetValue2(particle_component, "area_circle_radius", tether_length, tether_length+2)
    end
end

local function is_suitable_target(entity)
    return EntityGetIsAlive(entity) and not EntityHasTag(entity,"ew_notplayer")
end

function module.on_world_update_client()
    local host_playerdata = player_fns.peer_get_player_data(ctx.host_id, true)
    if host_playerdata == nil or not is_suitable_target(host_playerdata.entity) or not is_suitable_target(ctx.my_player.entity) then
        return
    end
    if GameGetFrameNum() % 10 == 7 then
        local x1, y1 = EntityGetTransform(host_playerdata.entity)
        local x2, y2 = EntityGetTransform(ctx.my_player.entity)
        local dx = x1-x2
        local dy = y1-y2
        local dist_sq = dx*dx + dy*dy
        if dist_sq > tether_length_2 * tether_length_2 and not_in_hm(x1, y1) and not_in_hm(x2, y2) then
            EntitySetTransform(ctx.my_player.entity, x1, y1)
        end
    end
end

return module