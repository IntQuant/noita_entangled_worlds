local rpc = net.new_rpc_namespace()

local tapion = {}

function rpc.set_helpless(anger)
    GlobalsSetValue( "HELPLESS_KILLS", tostring(anger) )
end

function tapion.on_world_update()
    if GameGetFrameNum() % 60 == 57 then
        rpc.set_helpless(tonumber(GlobalsGetValue( "HELPLESS_KILLS", "1" )))
    end
end

return tapion