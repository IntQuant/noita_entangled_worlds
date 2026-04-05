local ctx = {
    ready = false,
    lib = {},
    hook = {},
    proxy_opt = {},
    cap = {}, -- Capabilities
}

setmetatable(ctx.hook, {
    __index = function(_, _)
        return function() end
    end,
})

ctx.init = function()
    ctx.host_id = 0
    ctx.my_id = nil
    ctx.players = {}
    ctx.run_ended = false
    ctx.player_data_by_local_entity = {}
    ctx.item_prevent_localize = {}
    ctx.events = {}
    ctx.is_inventory_open = false
    ctx.rpc_peer_id = nil
    ctx.rpc_player_data = nil
    ctx.is_wand_pickup = false
    ctx.is_paused = false
    ctx.host_frame_num = 0
    ctx.is_texting = false
    ctx.stop_cam = false
    ctx.timings = ""
end

local function is_log_performance_enabled()
    return ModSettingGet("quant.ew.log_performance")
end

local function is_log_stutters_enabled()
    return ModSettingGet("quant.ew.log_stutters")
end


function ctx.finish()
    if is_log_performance_enabled() and string.len(ctx.timings) > 1 then
        print(string.sub(ctx.timings, 1, -2) .. "}")
    end
    ctx.timings = "{"
end

function ctx.add_hook(hook_name, system_name, fn)
    if rawget(ctx.hook, hook_name) == nil then
        local tbl = {}
        if is_log_performance_enabled() or is_log_stutters_enabled() then
            setmetatable(tbl, {
                __call = function(self, ...)
                    for _, entry in ipairs(self) do
                        local start_time = GameGetRealWorldTimeSinceStarted()
                        util.tpcall(entry.fn, ...)
                        local end_time = GameGetRealWorldTimeSinceStarted()
                        local delta = (end_time - start_time) * 1000000
                        ctx.timings = ctx.timings .. hook_name .. "/" .. entry.system_name .. ":" .. delta .. ","
                        local delta_ms = delta / 1000
                        local limit = 0.2
                        if entry.system_name == "ewext_init" then
                            limit = 2.2
                        end
                        if delta_ms > limit and is_log_stutters_enabled() then
                            GamePrint("Hook "..hook_name.." of system "..entry.system_name.." took way too long: "..delta_ms)
                        end
                    end
                end,
            })
        else
            setmetatable(tbl, {
                __call = function(self, ...)
                    for _, entry in ipairs(self) do
                        util.tpcall(entry.fn, ...)
                    end
                end,
            })
        end
        ctx.hook[hook_name] = tbl
    end
    table.insert(ctx.hook[hook_name], { fn = fn, system_name = system_name })
end

function ctx.dofile_and_add_hooks(path, system_name)
    --print("Loading " .. path)
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
    return ctx.dofile_and_add_hooks(
        "mods/quant.ew/files/system/" .. system_name .. "/" .. system_name .. ".lua",
        system_name
    )
end

return ctx
