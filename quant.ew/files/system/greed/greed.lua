util.prepend(
    "data/scripts/status_effects/effect_curse_radioactive.lua",
    'EntityLoad( "data/entities/misc/convert_radioactive_with_delay.xml", x,y )',
    'CrossCall("ew_greed", x,y)\n' .. 'EntityLoad( "data/entities/misc/convert_radioactive_with_delay.xml", x,y )'
)

local rpc = net.new_rpc_namespace()

function rpc.greed(x, y)
    EntityLoad("data/entities/misc/convert_radioactive_with_delay.xml", x, y)
end

util.add_cross_call("ew_greed", function(x, y)
    rpc.greed(x, y)
end)

return {}
