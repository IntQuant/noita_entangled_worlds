ModLuaFileAppend("data/scripts/biomes/mountain/mountain_left_stub.lua",
	"mods/quant.ew/files/system/Respawn_mountain/append.lua")
ModLuaFileAppend("data/scripts/perks/perk_reroll_init.lua",
	"mods/quant.ew/files/system/Respawn_mountain/Perk_reroll_init.lua")

module = {}
	

util.add_cross_call("ew_spawn_ankh_anchor", function(x, y)
	local entity_id = EntityCreateNew("")
	EntityAddTag(entity_id, "teleportable_NOT")
	EntityAddComponent(entity_id, "VelocityComponent")
	EntityAddComponent(entity_id, "SimplePhysicsComponent")
	EntityAddComponent(entity_id, "SpriteComponent", {
		_tags = "enabled_in_world,character",
		alpha = "1",
		image_file = "mods/quant.ew/files/system/respawn_mountain/sprites/ankh.png",
		next_rect_animation = "",
		rect_animation = "",
		offset_x = "4",
		offset_y = "15",
		z_index = "20"
	})
	EntityAddComponent(entity_id, "CollisionTriggerComponent", {
		width = "40",
		height = "40",
		radius = "64",
		destroy_this_entity_when_triggered = "0",
		required_tag = "player_unit"
	})
	EntityAddComponent(entity_id, "LuaComponent", {
		script_collision_trigger_hit = "mods/quant.ew/files/system/Respawn_mountain/respawn_checkpoint.lua",
		execute_every_n_frame = "1"
	})
	EntitySetTransform(entity_id, x, y)
end
)

util.add_cross_call("ew_respwan_local_player", function()
	ctx.run_ended = false
	if ctx.proxy_opt.game_mode == "shared_health" then
		GlobalsSetValue("ew_shared_hp", "4")
	end
	local player, damage_model_component, game_effect_component, found_workshop_guard, respawn_position_x, respawn_position_y, pos_x, pos_y, variable_storage_components, game_stats_component

	local respawn_messages = { "But you're better now.", "Let's get you fixed up.", "Take a deep breath.",
		"Oooh! What in blazes...?", "You feel as if you've had a Deja'vu"}
	damage_model_component = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "DamageModelComponent")
	player = ctx.my_player
	if player.entity == nil or damage_model_component == nil then
		GamePrintImportant("Error", "Missing DamageModelComponent")
	else
		ComponentSetValue2(EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "DamageModelComponent"),
			"wait_for_kill_flag_on_death", true)
		local _, max_hp = util.get_ent_health(ctx.my_player.entity)
		local cap = util.get_ent_health_cap(ctx.my_player.entity)

		util.set_ent_health(ctx.my_player.entity, { max_hp, max_hp })
		util.set_ent_health_cap(ctx.my_player.entity, cap)

		GameRegenItemActionsInPlayer(player.entity)
		game_effect_component = GetGameEffectLoadTo(player.entity, "BLINDNESS", true)
		if game_effect_component ~= nil then
			ComponentSetValue2(game_effect_component, "frames", 120)
		end

		found_workshop_guard = false
		respawn_position_x = tonumber(MagicNumbersGetValue("DESIGN_PLAYER_START_POS_X"))     -- 227 default
		respawn_position_y = tonumber(MagicNumbersGetValue("DESIGN_PLAYER_START_POS_Y"))     -- -85 default
		pos_x, pos_y = EntityGetTransform(player.entity)
		pos_y = pos_y - 4
		for _, enemy_id in pairs(EntityGetInRadiusWithTag(pos_x, pos_y, 96, "enemy")) do
			if EntityGetName(enemy_id) == "$animal_necromancer_shop" and not EntityHasTag(enemy_id, "polymorphable_NOT") then     -- check if stevari spawned
				found_workshop_guard = true
			end                                                                                                                   -- if	$animal_necromancer_shop and not polymorphable_NOT
		end                                                                                                                       -- for	enemy_id
		if not found_workshop_guard then
			variable_storage_components = EntityGetComponent(player.entity, "VariableStorageComponent")
			if variable_storage_components ~= nil and #variable_storage_components ~= 0 then
				for _, variable_storage_component in ipairs(variable_storage_components) do
					if ComponentGetValue2(variable_storage_component, "name") == "respawn_location" then
						pos_x = ComponentGetValue(variable_storage_component, "value_string")
						if pos_x ~= nil then
							respawn_position_x = tonumber(pos_x)
						end
						pos_y = ComponentGetValue(variable_storage_component, "value_int")
						if pos_y ~= nil then
							respawn_position_y = tonumber(pos_y)
						end
					end     -- if	respawn_location
				end         -- for	variable_storage_component
			end             -- if	variable_storage_components
		end                 -- if	not found_workshop_guard
		if not GameHasFlagRun("ending_game_completed") then
			EntitySetTransform(player.entity, respawn_position_x, respawn_position_y)
			EntityLoad("data/entities/misc/matter_eater.xml", respawn_position_x, respawn_position_y)
			GamePrintImportant("You have died.", respawn_messages[Random(1, #respawn_messages)])
			local x, y = EntityGetTransform(ctx.my_player.entity)
			EntityLoad("mods/quant.ew/files/system/respawn_mountain/entities/ankh_emitter.xml", x, y + 30)
		end     -- if	not ending_game_completed
	end         -- else	damage_model_components
end
)

return module
