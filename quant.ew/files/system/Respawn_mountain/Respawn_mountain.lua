ModLuaFileAppend("data/scripts/biomes/mountain/mountain_left_stub.lua", "mods/quant.ew/files/system/Respawn_mountain/append.lua")
ModLuaFileAppend("data/scripts/perks/perk_reroll_init.lua", "mods/quant.ew/files/system/Respawn_mountain/Perk_reroll_init.lua")



util.add_cross_call("ew_spawn_ankh_anchor", function(x, y)
    local entity_id = EntityCreateNew("")
	EntityAddTag(entity_id, "teleportable_NOT")
	EntityAddComponent(entity_id, "VelocityComponent")
	EntityAddComponent(entity_id, "SimplePhysicsComponent")
	EntityAddComponent(entity_id, "SpriteComponent", { 
		_tags="enabled_in_world,character",
		alpha="1",
		image_file="mods/quant.ew/files/system/respawn_mountain/sprites/ankh.png",
		next_rect_animation="",
		rect_animation="",
		offset_x="4",
		offset_y="15",
		z_index="20"
	})
	EntityAddComponent(entity_id, "CollisionTriggerComponent", {
		width="40",
		height="40",
		radius="64",
		destroy_this_entity_when_triggered="0",
		required_tag="player_unit"
	})
	EntityAddComponent(entity_id, "LuaComponent", {
		script_collision_trigger_hit="mods/quant.ew/files/system/Respawn_mountain/respawn_checkpoint.lua",
		execute_every_n_frame="1"
	})
	EntitySetTransform(entity_id, x, y)
end
)


return {}
