function collision_trigger(player_id)
	local rpx, rpy, found_respawn_data, variable_storage_components, respawn_position_x, respawn_position_y
	rpx, rpy = EntityGetTransform(GetUpdatedEntityID())

	rpx = tostring(rpx)
	rpy = tostring(rpy - 25)

	found_respawn_data = false
	variable_storage_components = EntityGetComponent(player_id, "VariableStorageComponent") -- get all variables saved in this player
	if variable_storage_components ~= nil and #variable_storage_components ~= 0 then
		for i, variable_storage_component in ipairs(variable_storage_components) do
			if ComponentGetValue(variable_storage_component, "name") == "respawn_location" then -- if there already exists a respawn_position_x
				found_respawn_data = true
				respawn_position_x = ComponentGetValue(variable_storage_component, "value_string") -- get x coordinate
				respawn_position_y = ComponentGetValue(variable_storage_component, "value_int") -- get y coordinate
				if respawn_position_x == nil or respawn_position_y == nil or           -- shouldn't be null
					respawn_position_x ~= rpx or respawn_position_y ~= rpy 
				then        -- if they are not the same spawn (if spawn point changed coords it will be considered a different one)
					ComponentSetValue(variable_storage_component, "value_string", rpx)
					ComponentSetValue(variable_storage_component, "value_int", rpy)
					if respawn_position_x == nil or respawn_position_y == nil or
						(tonumber(respawn_position_x) - tonumber(rpx)) ^ 2 + -- dist^2>10^2
						(tonumber(respawn_position_y) - tonumber(rpy)) ^ 2 > 100
					then
						GamePrintImportant("Checkpoint Set.", "You will respawn here when you die.")
					end
				end
				--		return
			end -- if	respawn_location
		end -- for	variable_storage_component
	end -- if	variable_storage_components
	if not found_respawn_data then
		EntityAddComponent(player_id, "VariableStorageComponent", { -- respawn position is local, this means someone with cheat engine or any memory editing software can edit their spawn point, we do not care about this. (cant respawn on pvp mode) I can fix this if wanted just lazy
			name = "respawn_location",
			value_string = rpx,
			value_int = rpy
		})
		GamePrintImportant("Checkpoint Set.", "You will respawn here when you die.")
	end
end
