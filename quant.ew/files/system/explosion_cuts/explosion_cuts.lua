local mod = {}

local alive = {}

local last = 0

function mod.on_local_player_spawn()
    if ctx.is_host then
        net.proxy_send("material_list", "")
    end
end

function mod.on_world_update_host()
    local n = EntitiesGetMaxID()
    for ent = last + 1, n do
        if EntityGetIsAlive(ent) then
            local proj = EntityGetFirstComponentIncludingDisabled(ent, "ProjectileComponent")
            if proj ~= nil and ComponentGetValue2(proj, "on_death_explode") then
                local x, y = EntityGetTransform(ent)
                alive[ent] = {x, y, ComponentObjectGetValue2(proj, "config_explosion", "explosion_radius"), ComponentObjectGetValue2(proj, "config_explosion", "max_durability_to_destroy")}
            end
        end
    end
    last = n
    for ent, data in pairs(alive) do
        if not EntityGetIsAlive(ent) then
            local inp = math.floor(data[1]) .. " " .. math.floor(data[2]) .. " " .. math.floor(data[3]) .. " " .. math.floor(data[4])
            net.proxy_send("cut_through_world_circle", inp)
            alive[ent] = nil
        end
    end
end

return mod