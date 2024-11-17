local rpc = net.new_rpc_namespace()
local potion = {}

function potion.enable_in_world(item)
    for _, com in ipairs(EntityGetAllComponents(item) or {}) do
        EntitySetComponentIsEnabled(item, com, true)
    end
    EntitySetComponentIsEnabled(item, EntityGetFirstComponentIncludingDisabled(item, "SpriteComponent", "enabled_in_hand"), false)
    EntitySetComponentIsEnabled(item, EntityGetFirstComponentIncludingDisabled(item, "ItemChestComponent"), false)
    if EntityGetParent(item) ~= 0 then
        EntityRemoveFromParent(item)
    end
end

function rpc.got_thrown(peer_id, vx, vy)
    local item = ctx.players[peer_id].entity
    potion.enable_in_world(item)
    if peer_id == ctx.my_player.peer_id then
        local phys_component = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "PhysicsBodyComponent")
        local px, py, pr, pvx, pvy, pvr = np.PhysBodyGetTransform(phys_component)
        np.PhysBodySetTransform(phys_component, px, py, pr, pvx + vx, pvy + vy, pvr)
    end
end

np.CrossCallAdd("ew_potion_mimic_throw", function(item, vx, vy)
    rpc.got_thrown(player_fns.get_player_data_by_local_entity_id(item).peer_id, vx, vy)
end)

np.CrossCallAdd("ew_potion_mimic_pickup", function()
    local inventory_state = player_fns.serialize_items(ctx.my_player)
    if inventory_state ~= nil then
        net.send_player_inventory(inventory_state)
    end
end)

function potion.on_world_update()
    if EntityHasTag(ctx.my_player.entity, "mimic_potion") then
        local effect
        for _, child in ipairs(EntityGetAllChildren(ctx.my_player.entity) or {}) do
            local com = EntityGetFirstComponentIncludingDisabled(child, "GameEffectComponent")
            if com ~= nil then
                local effect_name = ComponentGetValue2(com, "effect")
                if effect_name == "POLYMORPH" or effect_name == "POLYMORPH_RANDOM"
                        or effect_name == "POLYMORPH_CESSATION" or effect_name == "POLYMORPH_UNSTABLE" then
                    effect = com
                    break
                end
            end
        end
        if effect ~= nil then
            EntitySetComponentIsEnabled(ctx.my_player.entity, effect, EntityGetParent(ctx.my_player.entity) == 0)
        end
    end
    --if InputIsKeyJustDown(16) then --when "m" is pressed
    --    LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/potion_mimic/poly.xml")
    --end
end

return potion