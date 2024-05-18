local bitser = dofile_once("mods/quant.ew/files/lib/bitser.lua")
local pollnet = dofile_once("mods/quant.ew/files/lib/pollnet.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local util = dofile_once("mods/quant.ew/files/src/util.lua")

local reactor = pollnet.Reactor()

local net_handling = dofile_once("mods/quant.ew/files/src/net_handling.lua")
local net = {}

ctx.lib.net = net

function net.update()
    reactor:update()
end

local string_split = util.string_split

function net.init()
    local ready = false
    local addr = os.getenv("NP_NOITA_ADDR") or "127.0.0.1:21251"
    net.sock = pollnet.open_ws("ws://"..addr)
    reactor:run(function()
        local sock = net.sock
        while true do
          local msg_decoded = nil
          local msg = sock:await()
          if string.byte(msg, 1, 1) == 2 then
            local msg_l = string.sub(msg, 2)
            local res = string_split(msg_l, " ")
            if res[1] == "ready" then
              ready = true
            else
              msg_decoded = {
                kind = "proxy",
                key = res[1],
                value = res[2],
              }
            end
          elseif string.byte(msg, 1, 1) == 1 then
            local peer_id_b = {string.byte(msg, 2, 2+8-1)}
            local mult = 1
            local peer_id = 0
            for _, b in ipairs(peer_id_b) do
              peer_id = peer_id + b * mult
              mult = mult * 256
            end
            local msg_l = string.sub(msg, 2+8)
            local success, item = pcall(bitser.loads, msg_l)
            if success then
              msg_decoded = {
                kind = "mod",
                peer_id = peer_id,
                key = item.key,
                value = item.value,
              }
            else
              print("Could not deserialize: "..item)
            end
          else
            print("Unknown msg")
          end
          if msg_decoded ~= nil and net_handling[msg_decoded.kind] ~= nil and net_handling[msg_decoded.kind][msg_decoded.key] ~= nil then
            if ctx.ready or msg_decoded.kind ~= "mod" then
                util.tpcall(net_handling[msg_decoded.kind][msg_decoded.key], msg_decoded.peer_id, msg_decoded.value)
            end
            -- GamePrint("NetHnd: "..msg_decoded.kind.." "..msg_decoded.key)
          end
        end
    end)
    while not ready do
        reactor:update()
        pollnet.sleep_ms(100)
        --print("Waiting for connection...")
    end
        
end

local DEST_PROXY = 1
local DEST_BROADCAST = 2

local MOD_RELIABLE = 4

function net.send_internal(msg, dest, reliable)
  if reliable then
    dest = dest + MOD_RELIABLE
  end
  net.sock:send_binary(string.char(dest)..msg)
end

function net.send(key, value, reliable)
  local encoded_msg = bitser.dumps({
    key = key,
    value = value,
  })
  net.send_internal(encoded_msg, DEST_BROADCAST, reliable)
end

function net.proxy_send(key, value)
  net.send_internal(key.." "..value, DEST_PROXY)
end

function net.proxy_notify_game_over()
  net.proxy_send("game_over", 1)
end

function net.send_player_update(input_data, pos_data, current_slot)
  net.send("player", {
    i = input_data,
    p = pos_data,
    s = current_slot,
  })
end

function net.send_player_inventory(inventory_state)
  net.send("inventory", inventory_state, true)
end

function net.send_player_perks(perk_data)
  net.send("perks", perk_data, true)
end

function net.send_welcome()
  net.send("welcome", nil, true)
end

function net.send_enemy_data(enemy_data)
  net.send("enemy", enemy_data)
end

function net.send_world_data(world_data)
  net.send("world", world_data)
end

function net.send_host_player_info(player_info)
  net.send("host_player", player_info)
end

function net.send_fire(fire_info)
  net.send("fire", fire_info, true)
end

function net.send_make_global(item_data)
  net.send("item_global", item_data, true)
end

function net.send_localize(peer_id, item_id)
  net.send("item_localize", {peer_id, item_id}, true)
end

function net.send_localize_request(item_id)
  net.send("item_localize_req", item_id, true)
end

function net.send_item_upload(item_data)
  net.send("item_upload", item_data, true)
end

return net