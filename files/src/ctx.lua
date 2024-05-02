local ctx = {
    ready = false
}

ctx.init = function()
    ctx.my_id = nil
    ctx.players = {}
end

return ctx