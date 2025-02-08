local rpc = net.new_rpc_namespace()

local module = {}

local pending_requests = {}

rpc.opts_reliable()
function rpc.request_flag(flag)
    if ctx.is_host then
        local res = GameHasFlagRun(flag)
        GameAddFlagRun(flag)
        rpc.got_flag(flag, ctx.rpc_peer_id, not res or ctx.proxy_opt.duplicate)
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
        rpc.got_flag_slow(ctx.rpc_peer_id, not res or ctx.proxy_opt.duplicate, ent)
    end
end

rpc.opts_reliable()
rpc.opts_everywhere()
function rpc.got_flag_slow(peer_id, state, ent)
    if peer_id == ctx.my_id then
        if state then
            ewext.track(ent)
        else
            EntityKill(ent)
        end
    end
end

local last = 0
function module.on_world_update()
    local n = EntitiesGetMaxID()
    for ent = last + 1, n do
        if EntityGetIsAlive(ent) and not EntityHasTag(ent, "ew_des") and EntityGetRootEntity(ent) == ent then
            local f = EntityGetFilename(ent)
            local seed = EntityGetFirstComponentIncludingDisabled(ent, "PositionSeedComponent")
            local x, y = EntityGetTransform(ent)
            local lx, ly = math.floor(x / 64), math.floor(y / 64)
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
                or f == "data/entities/items/pickup/potion_empty.xml"
                or f == "data/entities/animals/chest_mimic.xml"
                or f == "data/entities/animals/chest_leggy.xml"
            then
                local flag = f .. ":" .. math.floor(x / 512) .. ":" .. math.floor(y / 512)
                ewext.notrack(ent)
                rpc.request_flag_slow(flag, ent)
            elseif
                (
                    f == "data/entities/props/physics_fungus.xml"
                    and (lx == -29 or lx == -28 or lx == -27)
                    and (ly == -20 or ly == -19)
                )
                or (f == "data/entities/props/physics_fungus_big.xml" and lx == -29 and ly == -20)
                or (f == "data/entities/props/physics_fungus_small.xml" and lx == -27 and ly == -19)
                or (f == "data/entities/items/pickup/evil_eye.xml" and lx == -39 and ly == -4)
            then
                local flag = f .. ":" .. lx .. ":" .. ly
                ewext.notrack(ent)
                rpc.request_flag_slow(flag, ent)
            elseif seed ~= nil then
                local flag = f .. ":" .. x .. ":" .. y
                ewext.notrack(ent)
                rpc.request_flag_slow(flag, ent)
            end
        end
    end
    last = n
end

rpc.opts_reliable()
rpc.opts_everywhere()
function rpc.request_moon_flag_slow(x, y, dark)
    if ctx.is_host then
        local flag = "ew_moon_spawn" .. ":" .. math.floor(x / 512) .. ":" .. math.floor(y / 512)
        local res = GameHasFlagRun(flag)
        GameAddFlagRun(flag)
        rpc.got_flag_moon_slow(ctx.rpc_peer_id, not res or ctx.proxy_opt.duplicate, x, y, dark)
    end
end

rpc.opts_reliable()
rpc.opts_everywhere()
function rpc.got_flag_moon_slow(peer_id, state, x, y, dark)
    if peer_id == ctx.my_id and state then
        if dark then
            EntityLoad("data/entities/items/pickup/sun/newsun_dark.xml", x, y)
        else
            EntityLoad("data/entities/items/pickup/sun/newsun.xml", x, y)
        end
    end
end

util.add_cross_call("ew_moon_spawn", function(x, y, dark)
    rpc.request_moon_flag_slow(x, y, dark)
end)

return module