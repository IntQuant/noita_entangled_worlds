local rpc = net.new_rpc_namespace()

local tapion = {}

local helpless = 1

local killer = 0

function rpc.set(anger, killers)
    GlobalsSetValue("HELPLESS_KILLS", tostring(anger))
    GlobalsSetValue("ULTIMATE_KILLER_KILLS", tostring(killers))
    helpless = anger
    killer = killers
end

function rpc.set_helpless(anger, killers)
    if ctx.is_host then
        GlobalsSetValue("HELPLESS_KILLS", tostring(helpless + anger))
        GlobalsSetValue("ULTIMATE_KILLER_KILLS", tostring(killer + killers))
        helpless = helpless + anger
        killer = killer + killers
        rpc.set(helpless, killer)
    end
end

function tapion.on_world_update()
    if GameGetFrameNum() % 60 == 57 then
        local anger = tonumber(GlobalsGetValue("HELPLESS_KILLS", "1"))
        local killers = tonumber(GlobalsGetValue("ULTIMATE_KILLER_KILLS", "0"))
        if anger > helpless or killers > killer then
            if ctx.is_host then
                rpc.set(anger, killers)
            else
                rpc.set_helpless(anger - helpless, killers - killer)
            end
            helpless = anger
            killer = killers
        end
    end
end

return tapion
