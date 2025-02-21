local meat = {}
local rpc = net.new_rpc_namespace()
local ran = false

function rpc.disable_meat()
    ran = true
    for _, v in ipairs(EntityGetWithTag("no_heal_in_meat_biome") or {}) do
        EntitySetComponentsWithTagEnabled(v, "effect_no_heal_in_meat_biome", false)
    end
end

function meat.on_world_update()
    if GameGetFrameNum() % 60 == 0 and not ran then
        local exists = false
        for _, v in ipairs(EntityGetWithTag("no_heal_in_meat_biome") or {}) do
            if
                ctx.my_player.entity == EntityGetRootEntity(v)
                and EntityGetFirstComponent(v, "GameEffectComponent") == nil
            then
                exists = true
                break
            end
        end
        if exists then
            rpc.disable_meat()
            ran = true
        end
    end
end

return meat
