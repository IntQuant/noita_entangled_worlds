local ctx = {
    ready = false,
    lib = {},
    hook = {},
    proxy_opt = {},
    cap = {}, -- Capabilities
}

setmetatable(ctx.hook, {
    __index = function (_, k)
        return function() end
    end
})

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
    ctx.is_wand_pickup = false
end

function ctx.dofile_and_add_hooks(path)
    print("Loading "..path)
    local result = dofile_once(path)
    for key, value in pairs(result) do
        if string.sub(key, 1, 3) == "on_" then
            local hook_name = key
            if rawget(ctx.hook, hook_name) == nil then
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

function ctx.load_system(system_name)
    return ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/"..system_name.."/"..system_name..".lua")
end

return ctx