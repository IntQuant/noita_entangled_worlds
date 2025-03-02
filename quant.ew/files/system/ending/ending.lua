local rpc = net.new_rpc_namespace()

local module = {}

util.replace_text_in(
    "data/entities/animals/boss_centipede/sampo.xml",
    "data/entities/animals/boss_centipede/ending/sampo_start_ending_sequence.lua",
    "mods/quant.ew/files/system/ending/ending_sequence.lua"
)
ModLuaFileAppend(
    "data/entities/animals/boss_centipede/boss_centipede_before_fight.lua",
    "mods/quant.ew/files/system/ending/update_fix.lua"
)
ModLuaFileAppend(
    "data/entities/animals/boss_centipede/boss_centipede_update.lua",
    "mods/quant.ew/files/system/ending/update_fix.lua"
)
util.replace_text_in("data/entities/animals/boss_centipede/boss_centipede_shield.lua", "player_unit", "ew_peer")

local function float()
    local character_data = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "CharacterDataComponent")
    ComponentSetValue2(character_data, "mVelocity", 0, -40)
end

rpc.opts_reliable()
rpc.opts_everywhere()
function rpc.gather_and_do_ending(x, y, sx, sy)
    local died = false
    if EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
        died = true
        local damage = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "DamageModelComponent")
        if damage ~= nil then
            ComponentSetValue2(damage, "ui_report_damage", false)
            ComponentSetValue2(damage, "hp", 2 ^ -38)
        end
        EntityInflictDamage(
            ctx.my_player.entity,
            1000000,
            "DAMAGE_CURSE",
            "revive",
            "NONE",
            0,
            0,
            GameGetWorldStateEntity()
        )
    end
    async(function()
        if died then
            wait(40)
        end
        net.proxy_send("reset_world", "")
        EntitySetTransform(ctx.my_player.entity, x, y)

        local entity = EntityCreateNew("totally_sampo")
        EntitySetTransform(entity, sx, sy)

        wait(10)
        EntitySetTransform(ctx.my_player.entity, x, y)
        wait(10)
        EntitySetTransform(ctx.my_player.entity, x, y)
        wait(10)

        EntitySetTransform(ctx.my_player.entity, x, y)
        float()

        -- Emulate the following script being called from LuaComponent
        local old_updated = GetUpdatedEntityID
        function GetUpdatedEntityID()
            return entity
        end

        dofile("data/entities/animals/boss_centipede/ending/sampo_start_ending_sequence.lua")

        GetUpdatedEntityID = old_updated
    end)
end

util.add_cross_call("ew_ending_sequence", function(sx, sy, sampo_ent)
    EntityKill(sampo_ent)
    local x, y = EntityGetTransform(ctx.my_player.entity)
    rpc.gather_and_do_ending(x, y, sx, sy)
end)

return module
