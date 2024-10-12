local rpc = net.new_rpc_namespace()

ModLuaFileAppend("data/scripts/biomes/temple_shared.lua", "mods/quant.ew/files/system/stevari/append.lua")
np.CrossCallAdd("ew_is_host", function()
    return ctx.is_host
end)

rpc.opts_everywhere()
function rpc.spawn_stevari(pos_x, pos_y)
    if( GlobalsGetValue( "TEMPLE_PEACE_WITH_GODS" ) == "1" ) then
        return
    end

    local guard_spawn_id = EntityGetClosestWithTag( pos_x, pos_y, "guardian_spawn_pos" )
    local guard_x = pos_x
    local guard_y = pos_y

    if( guard_spawn_id ~= 0  ) then
        guard_x, guard_y = EntityGetTransform( guard_spawn_id )

        EntityKill( guard_spawn_id )
    end

    EntityLoad( "data/entities/misc/spawn_necromancer_shop.xml", guard_x, guard_y )
end

np.CrossCallAdd("ew_spawn_stevari", function(x, y)
    rpc.spawn_stevari(x, y)
end)