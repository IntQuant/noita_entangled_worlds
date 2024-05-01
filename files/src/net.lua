local first = true
local pollnet = dofile_once("mods/quant.ew/files/lib/pollnet.lua")
local reactor = pollnet.Reactor()

local pending_events = {}

local function poll_until_open(sock)
    while true do
      sock:poll()
      local status = sock:status()
      if status == "open" then 
        return true 
      elseif status == "error" or status == "closed" then
        return false
      end
      coroutine.yield()
    end
  end

local net = {}

function net.get_events()
    reactor:update()
    local c_pending_events = pending_events
    pending_events = {}
    return c_pending_events
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
    reactor:run(function() 
        local sock = pollnet.open_ws("ws://127.0.0.1:41251")
        --poll_until_open(sock)
        while true do
          local msg = sock:await()
          print(msg)
          if string.byte(msg, 1, 1) == 2 then
            local msg_l = string.sub(msg, 2)
            local res = string_split(msg_l, " ")
            if res[1] == "seed" then
              print(res[1].." "..res[2])
              SetWorldSeed(tonumber(res[2]))
              SetRandomSeed(tonumber(res[2]), 141)
              ready = true
            end
          else
            print("Unknown msg")
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

return net