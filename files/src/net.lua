local bitser = dofile_once("mods/quant.ew/files/lib/bitser.lua")
local pollnet = dofile_once("mods/quant.ew/files/lib/pollnet.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")

local reactor = pollnet.Reactor()

local net_handling = dofile_once("mods/quant.ew/files/src/net_handling.lua")
local net = {}

function net.update()
    reactor:update()
end

local function string_split( s, splitter )
  local words = {};
  for word in string.gmatch( s, '([^'..splitter..']+)') do
      table.insert( words, word );
  end
  return words;
end

function net.init()
    local ready = false
    net.sock = pollnet.open_ws("ws://127.0.0.1:21251")
    reactor:run(function() 
        local sock = net.sock
        --poll_until_open(sock)
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
            local peer_id_l, peer_id_h = string.byte(msg, 2, 3)
            local peer_id = peer_id_l + peer_id_h * 256
            local msg_l = string.sub(msg, 4)
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
              net_handling[msg_decoded.kind][msg_decoded.key](msg_decoded.peer_id, msg_decoded.value)
            end
            -- GamePrint("NetHnd: "..msg_decoded.kind.." "..msg_decoded.key)
          end 
        end
        ready = true
    end)
    while not ready do
        reactor:update()
        pollnet.sleep_ms(100)
        --print("Waiting for connection...")
    end
        
end

function net.send(key, value, reliable) -- TODO reliability
  local encoded_msg = bitser.dumps({
    key = key,
    value = value,
  })
  net.sock:send_binary(encoded_msg)
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

return net