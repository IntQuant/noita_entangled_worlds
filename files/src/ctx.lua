local ctx = {
    ready = false
}

ctx.init = function()
    ctx.host_id = 0
    ctx.my_id = nil
    ctx.players = {}
    ctx.entity_by_remote_id = {}
    ctx.run_ended = false
    ctx.player_data_by_local_entity = {}
end

return ctx