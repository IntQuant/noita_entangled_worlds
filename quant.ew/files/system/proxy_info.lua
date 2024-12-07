local module = {}

function module.on_world_update()
    if GameGetFrameNum() % 30 ~= 6 then
        return
    end
    for peer_id, player_data in pairs(ctx.players) do
        local x, y = EntityGetTransform(player_data.entity)
        if x ~= nil and y ~= nil then
            net.proxy_send("peer_pos", peer_id.." "..x.." "..y)
        end
    end
end

return module