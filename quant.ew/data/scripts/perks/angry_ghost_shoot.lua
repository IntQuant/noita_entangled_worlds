dofile_once("data/scripts/lib/utilities.lua")

function wand_fired( wand_id )
	local projectile_velocity = 600

	local entity_id = GetUpdatedEntityID()
	local children = EntityGetAllChildren( entity_id )
	local ghost_ids = {}
	local root = EntityGetRootEntity(entity_id)
	local shooter
	if EntityHasTag(root, "ew_peer") and not EntityHasTag(root, "ew_notplayer") then
		shooter = EntityGetWithTag("player_unit")[1]
	else
		shooter = root
	end

	if ( children ~= nil ) then
		for i,v in ipairs( children ) do
			if EntityHasTag( v, "angry_ghost" ) then
				table.insert( ghost_ids, v )
			end
		end
	end

	if ( wand_id ~= nil ) and ( wand_id ~= NULL_ENTITY ) then
		for a,ghost_id in ipairs( ghost_ids ) do
			local pos_x, pos_y = EntityGetTransform( ghost_id )
			local comp_cd = EntityGetFirstComponent( ghost_id, "VariableStorageComponent", "angry_ghost_cooldown" )

			if ( comp_cd ~= nil ) then
				local cd = ComponentGetValue2( comp_cd, "value_int" )

				if ( cd == 0 ) then
					SetRandomSeed(pos_x + GameGetFrameNum(), pos_y)
					projectile_velocity = Random( 550, 750 )

					local x,y,dir = EntityGetTransform( wand_id )
					local comp = EntityGetFirstComponent( ghost_id, "VariableStorageComponent", "angry_ghost_projectile_memory" )
					local projectile = "data/entities/projectiles/deck/light_bullet.xml"

					if ( comp ~= nil ) then
						projectile = ComponentGetValue2( comp, "value_string" )
					end

					if ( #projectile == 0 ) then
						projectile = "data/entities/projectiles/deck/light_bullet.xml"
					end

					-- print( projectile )

					if ( #projectile > 0 ) then
						local vel_x = math.cos( 0 - dir ) * projectile_velocity
						local vel_y = 0 - math.sin( 0 - dir ) * projectile_velocity
						shoot_projectile( shooter, projectile, pos_x, pos_y, vel_x, vel_y)

						cd = 4
					end
				end

				ComponentSetValue2( comp_cd, "value_int", cd )
			end
		end
	end
end