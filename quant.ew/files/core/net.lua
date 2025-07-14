local bitser = dofile_once("mods/quant.ew/files/lib/bitser.lua")
local hex_table = dofile_once("mods/quant.ew/files/lib/hex.lua")

local net_handling = dofile_once("mods/quant.ew/files/core/net_handling.lua")
local net = {}

net.net_handling = net_handling

local string_split = util.string_split

net._rpc_inner = {
    rpcs = {},
    opts = {},
}
net.rpc = {}
net.connect_failed = false

local rpc_inner = net._rpc_inner

local rpc_base = {}

function rpc_base.opts_reliable()
    rpc_inner.opts.reliable = true
end

-- Also call rpc on client who initiated it.
function rpc_base.opts_everywhere()
    rpc_inner.opts.everywhere = true
end

function rpc_base:create_var(name, cb_fn)
    local var = {}
    local rpc_name = "_set_var_" .. name
    var.values = {}
    var.prev_values = {}
    self.opts_reliable()
    self.opts_everywhere()
    self[rpc_name] = function(new_value)
        var.prev_values[ctx.rpc_peer_id] = var.values[ctx.rpc_peer_id]
        var.values[ctx.rpc_peer_id] = new_value
        if cb_fn ~= nil then
            cb_fn(new_value)
        end
    end
    local set_rpc = self[rpc_name]
    function var.set(new_value)
        if new_value ~= var.values[ctx.my_id] then
            set_rpc(new_value)
        end
    end
    ctx.add_hook("on_should_send_updates", "net_vars", function()
        set_rpc(var.values[ctx.my_id])
    end)
    return var
end

local rpc_meta = {
    __newindex = function(t, k, v)
        table.insert(rpc_inner.rpcs, v)
        local index = #rpc_inner.rpcs
        if t._ew_id ~= nil then
            index = t._ew_id .. t._ew_index
            t._ew_index = t._ew_index + 1
            print("Created API rpc: " .. index)
        end
        local reliable = rpc_inner.opts.reliable == true
        local everywhere = rpc_inner.opts.everywhere == true
        rawset(t, k, function(...)
            net.send(index, { ... }, reliable)
            if everywhere then
                ctx.rpc_peer_id = ctx.my_id
                ctx.rpc_player_data = ctx.my_player
                v(...)
                ctx.rpc_peer_id = nil
                ctx.rpc_player_data = nil
            end
        end)
        net_handling.mod[index] = function(peer_id, args)
            ctx.rpc_peer_id = peer_id
            ctx.rpc_player_data = player_fns.peer_get_player_data(peer_id)
            if ctx.rpc_peer_id == nil or ctx.rpc_player_data == nil then
                --util.print_traceback()
                ctx.rpc_peer_id = nil
                ctx.rpc_player_data = nil
                return
            end
            v(unpack(args))
            ctx.rpc_peer_id = nil
            ctx.rpc_player_data = nil
        end
        rpc_inner.opts = {}
    end,
    __index = rpc_base,
}

function net.new_rpc_namespace()
    local ret = {}
    setmetatable(ret, rpc_meta)
    return ret
end

function net.new_rpc_namespace_with_id(id)
    local ret = {}
    ret._ew_id = id
    ret._ew_index = 0
    setmetatable(ret, rpc_meta)
    return ret
end

local function handle_message(msg)
    local msg_decoded
    if string.byte(msg, 1, 1) == 2 then
        local msg_l = string.sub(msg, 2)
        local res = string_split(msg_l, " ")
        msg_decoded = {
            kind = "proxy",
            key = res[1],
            value = res[2],
            value2 = res[3],
        }
    elseif string.byte(msg, 1, 1) == 1 then
        local peer_id_b = { string.byte(msg, 2, 2 + 8 - 1) }
        local peer_id = ""
        for _, b in ipairs(peer_id_b) do
            peer_id = hex_table[b + 1] .. peer_id
        end

        local msg_l = string.sub(msg, 2 + 8)
        local success, item = pcall(bitser.loads, msg_l)
        if success then
            msg_decoded = {
                kind = "mod",
                peer_id = peer_id,
                key = item.key,
                value = item.value,
            }
        else
            print("Could not deserialize: " .. item)
        end
    elseif string.byte(msg, 1, 1) == 3 then
        msg_decoded = {
            kind = "proxy",
            peer_id = nil,
            key = string.byte(msg, 2, 2),
            value = string.sub(msg, 3),
        }
    else
        print("Unknown msg")
    end
    if
        msg_decoded ~= nil
        and net_handling[msg_decoded.kind] ~= nil
        and net_handling[msg_decoded.kind][msg_decoded.key] ~= nil
        and (not ctx.run_ended or msg_decoded.key == "join_notify")
        and (ctx.ready or msg_decoded.kind ~= "mod")
    then
        util.tpcall(
            net_handling[msg_decoded.kind][msg_decoded.key],
            msg_decoded.peer_id,
            msg_decoded.value,
            msg_decoded.value2
        )
    end
end

function net.update()
    while true do
        local msg = ewext.netmanager_recv()
        if msg == nil then
            break
        end
        handle_message(msg)
    end
    ewext.netmanager_flush()
end

function net.init()
    local ok, res = util.tpcall(ewext.netmanager_connect)
    if not ok then
        net.connect_failed = true
        return
    end
    for _, opt in ipairs(res) do
        handle_message(opt)
    end
end

local DEST_PROXY = 1
local DEST_BROADCAST = 2
local DEST_FLAGS = 0
local MOD_RELIABLE = 4 -- 0b101

function net.send_internal(msg, dest, reliable)
    if reliable then
        dest = dest + MOD_RELIABLE
    end
    ewext.netmanager_send(string.char(dest) .. msg)
end

function net.send_flags(flags)
    local dest = DEST_FLAGS + MOD_RELIABLE
    ewext.netmanager_send(string.char(dest) .. flags)
end

function net.send(key, value, reliable)
    local encoded_msg = bitser.dumps({
        key = key,
        value = value,
    })
    net.send_internal(encoded_msg, DEST_BROADCAST, reliable)
end

function net.estimate_rpc_size(message)
    local encoded_msg = bitser.dumps({
        key = 42,
        value = message,
    })
    return string.len(encoded_msg)
end

function net.send_to_host(key, value, reliable)
    net.send(key, value, reliable) -- TODO actually only send to host
end

function net.proxy_send(key, value)
    net.send_internal(key .. " " .. value, DEST_PROXY)
end

function net.proxy_notify_game_over()
    net.proxy_send("game_over", 1)
end

function net.send_player_inventory(inventory_state, spells)
    net.send("inventory", { inventory_state, spells }, true)
end

function net.send_player_perks(perk_data)
    net.send("perks", perk_data, true)
end

function net.send_welcome()
    net.send("welcome", nil, true)
end

function net.send_fire(fire_info)
    net.send("fire", fire_info, true)
end

return net
