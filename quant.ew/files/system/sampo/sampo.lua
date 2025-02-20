local sampo = {}
local rpc = net.new_rpc_namespace()
local first = true
function rpc.spawn_sampo()
    if ctx.is_host then
        local x, y = EntityGetTransform(ctx.my_player.entity)
        EntityLoad("data/entities/animals/boss_centipede/sampo.xml", x, y)
    end
    first = false
end
function sampo.on_world_update_client()
    if ctx.proxy_opt.host_sampo and first and GameGetFrameNum() % 10 == 3 then
        for _, ent in ipairs(GameGetAllInventoryItems(ctx.my_player.entity) or {}) do
            if EntityHasTag(ent, "this_is_sampo") then
                EntityKill(ent)
                rpc.spawn_sampo()
                first = false
            end
        end
    end
end
return sampo
