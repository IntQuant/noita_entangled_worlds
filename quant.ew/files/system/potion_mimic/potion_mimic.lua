local rpc = net.new_rpc_namespace()
local potion = {}

function potion.enable_in_world(item)
    EntitySetComponentsWithTagEnabled(item, "enabled_in_hand", false)
    EntitySetComponentsWithTagEnabled(item, "enabled_in_inventory", false)
    EntitySetComponentsWithTagEnabled(item, "enabled_in_world", true)
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

util.add_cross_call("ew_potion_mimic_throw", function(item, vx, vy)
    rpc.got_thrown(player_fns.get_player_data_by_local_entity_id(item).peer_id, vx, vy)
end)

util.add_cross_call("ew_potion_mimic_pickup", function()
    local inventory_state = player_fns.serialize_items(ctx.my_player)
    if inventory_state ~= nil then
        net.send_player_inventory(inventory_state)
    end
end)

function rpc.ensure_held(peer_id)
    if peer_id == ctx.my_player.peer_id then
        local ent = ctx.my_player.entity
        local inv
        for _, child in ipairs(EntityGetAllChildren(ent) or {}) do
            if EntityGetName( child ) == "inventory_quick" then
                inv = child
                break
            end
        end
        local has_player = false
        for _, child in ipairs(EntityGetAllChildren(inv) or {}) do
            local player = player_fns.get_player_data_by_local_entity_id(child)
            if player ~= nil and player.peer_id == ctx.rpc_player_data.peer_id then
                has_player = true
            end
        end
        if not has_player then
            EntityAddChild(inv, ctx.rpc_player_data.entity)
        end
    end
end

function potion.on_world_update()
    if EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "ItemComponent") ~= nil then
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
        local root = EntityGetRootEntity(ctx.my_player.entity)
        if effect ~= nil then
            EntitySetComponentIsEnabled(ctx.my_player.entity, effect, root == ctx.my_player.entity)
        end
        if GameGetFrameNum() % 60 == 53 and root ~= ctx.my_player.entity then
            rpc.ensure_held(player_fns.get_player_data_by_local_entity_id(root).peer_id)
        end
    end
    --if InputIsKeyJustDown(16) then --when "m" is pressed
    --    LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/potion_mimic/poly.xml")
    --end
end

return potion