local sampo = {}
local rpc = net.new_rpc_namespace()
local first = true
function rpc.spawn_sampo()
    if ctx.is_host then
        local x, y = EntityGetTransform(ctx.my_player.entity)
        EntityLoad("data/entities/animals/boss_centipede/sampo.xml", x, y)
    end
    first = false
end

-- rpc.opts_reliable()
-- function rpc.activate_boss_centipede_fight()
-- 	if not ctx.is_host then
-- 		for _,entity_id in pairs(entities) do
-- 			EntitySetComponentsWithTagEnabled( entity_id, "disabled_at_start", true )
-- 			EntitySetComponentsWithTagEnabled( entity_id, "enabled_at_start", false )
-- 			PhysicsSetStatic( entity_id, false )

-- 			--activate Kolmisilmä locally
-- 			if EntityHasTag( entity_id, "boss_centipede" ) then
-- 				EntityAddTag( entity_id, "boss_centipede_active" )
				
-- 				local child_entities = EntityGetAllChildren( entity_id )
-- 				local child_to_remove = 0
				
-- 				if ( child_entities ~= nil ) then
-- 					for child_id in  child_entities do
-- 						-- fix
-- 						if EntityHasTag( child_id, "protection" ) then
-- 							child_to_remove = child_id
-- 						end
-- 					end
-- 				end
				
-- 				if ( child_to_remove ~= 0 ) then
-- 					EntityKill( child_to_remove )
-- 				end
-- 			end
-- 		end
-- 	end
	
-- end

function sampo.on_world_update_client()

	-- give sampo to host
    if ctx.proxy_opt.host_sampo and first and GameGetFrameNum() % 10 == 3 then
        for _, ent in ipairs(GameGetAllInventoryItems(ctx.my_player.entity) or {}) do
            if EntityHasTag(ent, "this_is_sampo") then
                EntityKill(ent)
                rpc.spawn_sampo()
                first = false
            end
        end
    end
end

return sampo
