local rpc = net.new_rpc_namespace()

local module = {}

local pending_requests = {}

rpc.opts_reliable()
function rpc.request_flag(flag)
    if ctx.is_host then
        local res = GameHasFlagRun(flag)
        GameAddFlagRun(flag)
        rpc.got_flag(flag, ctx.rpc_peer_id, not res)
    end
end

rpc.opts_reliable()
function rpc.got_flag(flag, peer_id, state)
    if peer_id == ctx.my_id then
        local coro = pending_requests[flag]
        if coro ~= nil then
            coroutine.resume(coro, state)
        end
    end
end

function module.request_flag(flag)
    if ctx.is_host then
        local res = GameHasFlagRun(flag)
        GameAddFlagRun(flag)
        return not res
    end
    
    local current = coroutine.running()
    pending_requests[flag] = current
    rpc.request_flag(flag)
    return coroutine.yield()
end

return module
