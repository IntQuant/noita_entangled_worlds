local np = require("noitapatcher")
local rpc = net.new_rpc_namespace()

function rpc.spawn_gate()
    local x, y = EntityGetTransform(ctx.my_player.entity)
    for _, ent in ipairs(EntityGetInRadius(x, y, 1024) or {}) do
        if EntityGetFilename(ent) == "data/entities/buildings/wizardcave_gate.xml" then
            local x_orig, y_orig = EntityGetTransform( ent )
            -- spawn monster spawner
            EntityLoad("mods/quant.ew/files/system/gate_boss/wizardcave_gate_monster_spawner.xml", x_orig, y_orig)
            -- audio
            GamePlaySound( "data/audio/Desktop/projectiles.snd", "player_projectiles/crumbling_earth/create", x_orig, y_orig )
            GameTriggerMusicFadeOutAndDequeueAll( 3.0 )
            GameTriggerMusicEvent( "music/oneshot/04", true, x_orig, y_orig )
            -- remove self
            EntityKill(ent)
            return
        end
    end
end

util.add_cross_call("ew_spawn_gate", function()
    rpc.spawn_gate()
end)

return {}