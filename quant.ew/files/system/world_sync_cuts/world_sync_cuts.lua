util.add_cross_call("ew_cut_through_world", function(x, min_y, max_y, radius, _radius2)
    if ctx.is_host then
        net.proxy_send("cut_through_world", x .. " " .. min_y .. " " .. max_y .. " " .. radius)
    end
end)

ModLuaFileAppend(
    "data/scripts/magic/beam_from_sky.lua",
    "mods/quant.ew/files/system/world_sync_cuts/patch_cut_through_world.lua"
)

local module = {}

return module
