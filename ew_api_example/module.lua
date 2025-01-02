local ew_api = dofile_once("mods/quant.ew/files/api/ew_api.lua")

-- Needs a unique, but preferably short identifier.
local rpc = ew_api.new_rpc_namespace("api_example")

-- Make the next rpc be delivered reliably.
-- This means that it will be called exactly once (unless a disconnection happens), and will be ordered the same way.
-- E. g. if you call rpc.rpc1(), rpc.rpc1(), rpc.rpc2() that are reliable, they will get called in the same order on other clients.
rpc.opts_reliable()
-- This rpc will also get called locally.
rpc.opts_everywhere()
function rpc.send_hi()
    GamePrint("Hi from " .. ew_api.rpc_player_data().name)
end

local module = {}

function module.on_world_update()
    if GameGetFrameNum() % 60 == 0 then
        GamePrint("Hi from api example!")
        rpc.send_hi()
    end
end

return module
