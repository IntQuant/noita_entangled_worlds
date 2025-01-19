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

rpc.opts_reliable()
rpc.opts_everywhere()
function rpc.request_flag_slow(flag, ent)
    if ctx.is_host then
        local res = GameHasFlagRun(flag)
        GameAddFlagRun(flag)
        rpc.got_flag_slow(ctx.rpc_peer_id, res, ent)
    end
end

rpc.opts_reliable()
rpc.opts_everywhere()
function rpc.got_flag_slow(peer_id, state, ent)
    if peer_id == ctx.my_id then
        if state then
            EntityKill(ent)
        else
            ewext.track(ent)
        end
    end
end

local last = 0
function module.on_world_update()
    local n = EntitiesGetMaxID()
    for ent = last + 1, n do
        if EntityGetIsAlive(ent) and not EntityHasTag(ent, "ew_des") then
            local f = EntityGetFilename(ent)
            if
                f == "data/entities/misc/orb_07_pitcheck_b.xml"
                or f == "data/entities/misc/orb_07_pitcheck_a.xml"
                or f == "data/entities/buildings/maggotspot.xml"
                or f == "data/entities/buildings/essence_eater.xml"
                or f == "data/entities/props/music_machines/music_machine_00.xml"
                or f == "data/entities/props/music_machines/music_machine_01.xml"
                or f == "data/entities/props/music_machines/music_machine_02.xml"
                or f == "data/entities/props/music_machines/music_machine_03.xml"
                or f == "data/entities/animals/boss_fish/fish_giga.xml"
            then
                local x, y = EntityGetTransform(ent)
                local flag = f .. ":" .. math.floor(x / 512) .. ":" .. math.floor(y / 512)
                ewext.notrack(ent)
                rpc.request_flag_slow(flag, ent)
            end
        end
    end
    last = n
end

return module
