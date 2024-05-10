local ctx = {
    ready = false
}

ctx.init = function()
    ctx.host_id = 0
    ctx.my_id = nil
    ctx.players = {}
    ctx.entity_by_remote_id = {}
end

return ctx