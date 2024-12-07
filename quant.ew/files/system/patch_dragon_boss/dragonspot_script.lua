dofile_once("data/scripts/lib/utilities.lua")
function collision_trigger()

    local entity_id    = GetUpdatedEntityID()
    local pos_x, pos_y = EntityGetTransform( entity_id )
    if GameHasFlagRun("ew_flag_this_is_host") then
        local eid = EntityLoad( "data/entities/animals/boss_dragon.xml", pos_x, pos_y )
        EntityAddComponent(eid, "LuaComponent", { script_death = "data/scripts/animals/boss_dragon_death.lua",
                    execute_every_n_frame = "-1"})
    end
    EntityLoad( "data/entities/particles/image_emitters/magical_symbol_fast.xml", pos_x, pos_y )

    EntityKill( entity_id )
end