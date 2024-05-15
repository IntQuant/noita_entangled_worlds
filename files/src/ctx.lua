local ctx = {
    ready = false,
    lib = {},
}

ctx.init = function()
    ctx.host_id = 0
    ctx.my_id = nil
    ctx.players = {}
    ctx.entity_by_remote_id = {}
    ctx.run_ended = false
    ctx.player_data_by_local_entity = {}
    ctx.item_prevent_localize = {}
    ctx.events = {}
    ctx.is_inventory_open = false
end

return ctx