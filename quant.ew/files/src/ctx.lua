local ctx = {
    ready = false,
    lib = {},
    hook = {},
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
    ctx.rpc_peer_id = nil
end

function ctx.dofile_and_add_hooks(path)
    local result = dofile_once(path)
    for key, value in pairs(result) do
        if string.sub(key, 1, 3) == "on_" then
            local hook_name = key
            if ctx.hook[hook_name] == nil then
                local tbl = {}
                setmetatable(tbl, {
                    __call = function (self, ...)
                        for _, fn in ipairs(self) do
                            fn(...)
                        end
                    end
                })
                ctx.hook[hook_name] = tbl
            end
            table.insert(ctx.hook[hook_name], value)
        end
    end
    return result
end

return ctx