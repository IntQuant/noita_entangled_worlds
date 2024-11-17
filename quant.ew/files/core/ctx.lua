local ctx = {
    ready = false,
    lib = {},
    hook = {},
    proxy_opt = {},
    cap = {}, -- Capabilities
}

setmetatable(ctx.hook, {
    __index = function (_, _)
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
    ctx.host_frame_num = 0
end

local function is_measure_perf_enabled()
    -- return ctx.proxy_opt.debug
    return false
end

function ctx.add_hook(hook_name, system_name, fn)
    if rawget(ctx.hook, hook_name) == nil then
        local tbl = {}
        if is_measure_perf_enabled() then
            setmetatable(tbl, {
                __call = function (self, ...)
                    for _, entry in ipairs(self) do
                        local start_time = GameGetRealWorldTimeSinceStarted()
                        util.tpcall(entry.fn, ...)
                        local end_time = GameGetRealWorldTimeSinceStarted()
                        local delta = (end_time-start_time) * 1000
                        if delta > 0.02 then
                            print("Hook "..hook_name.." took "..(delta).." ms to run for "..entry.system_name)
                        end
                    end
                end
            })
        else
            setmetatable(tbl, {
                __call = function (self, ...)
                    for _, entry in ipairs(self) do
                        util.tpcall(entry.fn, ...)
                    end
                end
            })
        end
        ctx.hook[hook_name] = tbl
    end
    table.insert(ctx.hook[hook_name], {fn=fn, system_name=system_name})
end

function ctx.dofile_and_add_hooks(path, system_name)
    print("Loading "..path)
    system_name = system_name or path
    local result = dofile_once(path)
    for hook_name, fn in pairs(result) do
        if string.sub(hook_name, 1, 3) == "on_" then
            ctx.add_hook(hook_name, system_name, fn)
        end
    end
    return result
end

function ctx.load_system(system_name)
    return ctx.dofile_and_add_hooks("mods/quant.ew/files/system/"..system_name.."/"..system_name..".lua", system_name)
end

return ctx