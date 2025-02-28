local rpc = net.new_rpc_namespace()

util.add_cross_call("ew_cut_through_world", function(x, min_y, max_y, radius, _radius2)
    if ctx.is_host then
        net.proxy_send("cut_through_world", x .. " " .. min_y .. " " .. max_y .. " " .. radius)
    end
end)

util.prepend(
    "data/scripts/items/beamstone_kick.lua",
    'EntityLoad( "data/entities/misc/beam_from_sky.xml", pos_x, pos_y )',
    'local util = dofile_once("mods/quant.ew/files/resource/util_min.lua")\n'
        .. "if util.do_i_own(entity_id) then\n"
        .. 'CrossCall("ew_beam", pos_x, pos_y)\n'
        .. "end"
)

rpc.opts_everywhere()
function rpc.beam(x, y)
    EntityLoad("data/entities/misc/beam_from_sky.xml", x, y)
end

util.add_cross_call("ew_beam", function(x, y)
    rpc.beam(x, y)
end)

ModLuaFileAppend(
    "data/scripts/magic/beam_from_sky.lua",
    "mods/quant.ew/files/system/world_sync_cuts/patch_cut_through_world.lua"
)

return {}
