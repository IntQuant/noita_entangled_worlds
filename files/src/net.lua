local first = true
local pollnet = dofile_once("mods/quant.ew/files/lib/pollnet.lua")
local reactor = pollnet.Reactor()

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

local net = {
    get_events = function()
        if first then
            first = false
            return {
                { kind = "connect", peer_id = 1}
            }
        end
        return {}
    end
}

function net.init()
    local ready = false
    reactor:run(function() 
        local sock = pollnet.open_ws("ws://127.0.0.1:41251")
        poll_until_open(sock)
        ready = true
    end)
    while not ready do
        reactor:update()
        pollnet.sleep_ms(100)
        print("Waiting for connection...")
    end
        
end

return net