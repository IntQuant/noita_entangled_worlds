local uniq_flags = dofile_once("mods/quant.ew/files/system/uniq_flags/uniq_flags.lua")

ModLuaFileAppend(
    "data/scripts/buildings/dragonspot.lua",
    "mods/quant.ew/files/system/patch_dragon_boss/dragonspot_script.lua"
)
util.replace_text_in("data/entities/buildings/dragonspot.xml", "player_unit", "ew_peer")

local module = {}

local function spawn_dragon_boss(entity_id)
    local pos_x, pos_y = EntityGetTransform(entity_id)

    local eid = EntityLoad("data/entities/animals/boss_dragon.xml", pos_x, pos_y)
    EntityAddComponent(
        eid,
        "LuaComponent",
        { script_death = "data/scripts/animals/boss_dragon_death.lua", execute_every_n_frame = "-1" }
    )

    EntityLoad("data/entities/particles/image_emitters/magical_symbol_fast.xml", pos_x, pos_y)
end

util.add_cross_call("ew_spawn_dragon_boss", function(entity_id)
    local pos_x, pos_y = EntityGetTransform(entity_id)
    async(function()
        if uniq_flags.request_flag("spawn_dragon_boss_" .. pos_x .. "_" .. pos_y) then
            spawn_dragon_boss(entity_id)
        end
        EntityKill(entity_id)
    end)
end)

return module
