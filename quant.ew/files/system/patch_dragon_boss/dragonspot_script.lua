dofile_once("data/scripts/lib/utilities.lua")

function collision_trigger()
    if not GameHasFlagRun("ew_flag_this_is_host") then
        return
    end

	local entity_id    = GetUpdatedEntityID()
	local pos_x, pos_y = EntityGetTransform( entity_id )
	EntityLoad( "mods/quant.ew/files/system/patch_dragon_boss/dragon_boss_extra.xml", pos_x, pos_y )
	EntityLoad( "data/entities/particles/image_emitters/magical_symbol_fast.xml", pos_x, pos_y )

	EntityKill( entity_id )
end