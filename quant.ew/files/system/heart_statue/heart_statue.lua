local rpc = net.new_rpc_namespace()

local heart_statue = {}

-- Manually remove item from the inventory
local function enable_in_world(item)
    EntitySetComponentsWithTagEnabled(item, "enabled_in_hand", false)
    EntitySetComponentsWithTagEnabled(item, "enabled_in_inventory", false)
    EntitySetComponentsWithTagEnabled(item, "enabled_in_world", true)
    if EntityGetParent(item) ~= 0 then
        EntityRemoveFromParent(item)
    end
end

rpc.opts_everywhere()
function rpc.got_thrown(peer_id, phys_transform)
    local item = ctx.players[peer_id].entity
    enable_in_world(item)
    if peer_id == ctx.my_player.peer_id then
        local phys_component = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "PhysicsBodyComponent")
        local t = phys_transform
        np.PhysBodySetTransform(phys_component, t.px, t.py, t.pr, t.pvx, t.pvy, t.pvr)
    end
end

util.add_cross_call("ew_heart_statue_throw", function(item)
    -- Transfering physic transform from the player that throws the item to other players
    local phys_component = EntityGetFirstComponentIncludingDisabled(item, "PhysicsBodyComponent")
    local px, py, pr, pvx, pvy, pvr = np.PhysBodyGetTransform(phys_component)
    rpc.got_thrown(
        player_fns.get_player_data_by_local_entity_id(item).peer_id,
        { px = px, py = py, pr = pr, pvx = pvx, pvy = pvy, pvr = pvr }
    )
end)

util.add_cross_call("ew_heart_statue_pickup", function()
    local inventory_state = player_fns.serialize_items(ctx.my_player)
    if inventory_state ~= nil then
        net.send_player_inventory(inventory_state)
    end
end)

function rpc.ensure_held(peer_id)
    if peer_id ~= ctx.my_player.peer_id then
        return
    end
    local ent = ctx.my_player.entity
    local inv
    for _, child in ipairs(EntityGetAllChildren(ent) or {}) do
        if EntityGetName(child) == "inventory_quick" then
            inv = child
            break
        end
    end
    if inv == nil then
        return
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

function heart_statue.on_world_update()
    if not EntityHasTag(ctx.my_player.entity, "heart_statue") then
        return
    end

    -- Ensure polymorph is always enabled
    local effect
    for _, child in ipairs(EntityGetAllChildren(ctx.my_player.entity) or {}) do
        local com = EntityGetFirstComponentIncludingDisabled(child, "GameEffectComponent")
        if com ~= nil then
            local effect_name = ComponentGetValue2(com, "effect")
            if
                effect_name == "POLYMORPH"
                or effect_name == "POLYMORPH_RANDOM"
                or effect_name == "POLYMORPH_CESSATION"
                or effect_name == "POLYMORPH_UNSTABLE"
            then
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
        local data = player_fns.get_player_data_by_local_entity_id(root)
        if data ~= nil then
            rpc.ensure_held(data.peer_id)
        end
    end
end

return heart_statue

