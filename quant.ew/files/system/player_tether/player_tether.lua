local util = dofile_once("mods/quant.ew/files/core/util.lua")
local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/core/player_fns.lua")

local tether_length = ctx.proxy_opt.tether_length
local tether_length_2 = tether_length + 200

local module = {}

function module.on_client_spawned(peer_id, new_playerdata)
    local is_host = peer_id == ctx.host_id
    if is_host then
        local zone_ent = EntityLoad("mods/quant.ew/files/system/player_tether/zone_entity.xml")
        EntityAddChild(new_playerdata.entity, zone_ent)
        local particle_component = EntityGetFirstComponentIncludingDisabled(zone_ent, "ParticleEmitterComponent")
        ComponentSetValue2(particle_component, "area_circle_radius", tether_length, tether_length+2)
    end
end

function module.on_world_update_client()
    local host_playerdata = player_fns.peer_get_player_data(ctx.host_id, true)
    if host_playerdata == nil then
        return
    end
    if GameGetFrameNum() % 10 == 7 then
        local x1, y1 = EntityGetTransform(host_playerdata.entity)
        local x2, y2 = EntityGetTransform(ctx.my_player.entity)
        local dx = x1-x2
        local dy = y1-y2
        local dist_sq = dx*dx + dy*dy
        if dist_sq > tether_length_2 * tether_length_2 then
            EntitySetTransform(ctx.my_player.entity, x1, y1)
        end
    end
end

return module
