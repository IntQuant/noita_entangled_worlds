local rpc = net.new_rpc_namespace()
local boss = {}

function rpc.run_hp(hpp)
    local kivi = EntityGetWithTag("boss")
    local has = false
    for _, ent in ipairs(kivi or {}) do
        if EntityGetName(ent) == "$animal_boss_sky" then
            has = true
            break
        end
    end
    if not has then
        local hp, max_hp = util.get_ent_health(ctx.my_player.entity)
        local player_hp_percentage = hp / max_hp
        if player_hp_percentage >= hpp then
            local damage = math.max(0, max_hp * (player_hp_percentage - hpp))
            if damage > 0 and not EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
                EntityInflictDamage(
                    ctx.my_player.entity,
                    damage,
                    "DAMAGE_HOLY",
                    "$damage_holy",
                    "NONE",
                    0,
                    0,
                    NULL_ENTITY
                )
                local px, py, _, _, _ = EntityGetTransform(ctx.my_player.entity)
                local hx, hy = EntityGetHotspot(ctx.my_player.entity, "cape_root", false)
                local tx, ty = px + hx, py + hy - 2
                EntityLoad("data/entities/particles/poof_red_tiny.xml", tx, ty)
            end
        end
    end
end

util.add_cross_call("ew_kivi", function(hp_percentage)
    rpc.run_hp(hp_percentage)
end)

function rpc.run_deer(dmg)
    local deer = EntityGetWithTag("islandspirit")
    if #deer == 0 then
        if not EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
            EntityInflictDamage(
                ctx.my_player.entity,
                dmg,
                "DAMAGE_CURSE",
                "$animal_islandspirit",
                "DISINTEGRATED",
                0,
                0,
                GameGetWorldStateEntity()
            )
        end
    end
end

util.add_cross_call("ew_deer", function(dmg)
    rpc.run_deer(dmg)
end)

return boss
