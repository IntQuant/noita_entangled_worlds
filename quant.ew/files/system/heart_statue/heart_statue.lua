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

local function add_legs(item)
    local child_entities = EntityGetAllChildren(item)
    if child_entities ~= nil then
        for _, child_entity in ipairs(child_entities) do
            local child_entity_name = EntityGetName(child_entity)
            if child_entity_name == "hs_leg" then
                EntityRemoveFromParent(child_entity)
                EntityKill(child_entity)
            end
        end
    end

    async(function()
        wait(1)
        util.add_player_cape_for_fun(item)

        local x, y = EntityGetTransform(item)
        for i = 1, 5 do
            local leg = EntityLoad("mods/quant.ew/files/system/heart_statue/heart_statue_leg.xml", x, y)
            EntityAddChild(item, leg)
        end
    end)
end

rpc.opts_everywhere()
function rpc.got_thrown(peer_id, phys_transform)
    local item = ctx.players[peer_id].entity
    enable_in_world(item)

    local child_entities = EntityGetAllChildren(item)
    if child_entities ~= nil then
        for _, child_entity in ipairs(child_entities) do
            local child_entity_name = EntityGetName(child_entity)
            if child_entity_name == "hs_leg" then
                EntityRemoveFromParent(child_entity)
                EntityKill(child_entity)
            end
        end
    end

    add_legs(item)
    async(function()
        wait(1)
        util.add_player_cape_for_fun(item)

        local x, y, rot, scale_x, scale_y = EntityGetTransform(item)
        EntitySetTransform(item, x, y, 0, math.abs(scale_x), math.abs(scale_y))
    end)

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

util.add_cross_call("ew_heart_statue_pickup", function(item)
    if item then
        util.remove_cape(item)
    end

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

local function get_closest_real_player(entity_id)
    local x, y = EntityGetTransform(entity_id)
    local players = EntityGetWithTag("ew_peer") or {}
    local closest_player
    local min_dist = 999999

    for _, p_id in ipairs(players) do
        if not EntityHasTag(p_id, "polymorphed_player") then
            local px, py = EntityGetTransform(p_id)
            local dist = ((px - x) ^ 2 + (py - y) ^ 2)
            if dist < min_dist then
                min_dist = dist
                closest_player = p_id
            end
        end
    end

    return closest_player
end

local last_movement_dir = 1
local last_x, last_y = 0, 0
local frames_little_change = 0

function heart_statue.on_world_update()
    local entity_id = ctx.my_player.entity
    if not EntityHasTag(entity_id, "heart_statue") then
        return
    end

    -- Ensure polymorph is always enabled
    local effect
    for _, child in ipairs(EntityGetAllChildren(entity_id) or {}) do
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
    local root = EntityGetRootEntity(entity_id)
    if effect ~= nil then
        EntitySetComponentIsEnabled(entity_id, effect, root == entity_id)
    end

    if root ~= entity_id then
        if GameGetFrameNum() % 60 == 53 then
            util.remove_cape(entity_id)

            local data = player_fns.get_player_data_by_local_entity_id(root)
            if data ~= nil then
                rpc.ensure_held(data.peer_id)
            end
        end
    else
        local ik_animator = EntityGetFirstComponent(entity_id, "IKLimbsAnimatorComponent")
        local any_leg_attached = false
        if ik_animator ~= nil then
            any_leg_attached = ComponentGetValue2(ik_animator, "mHasGroundAttachmentOnAnyLeg")
        end

        local x, y, rot, scale_x, scale_y = EntityGetTransform(entity_id)
        local force_magnitude
        local px, py
        local dx, dy
        local dist = math.abs(last_x - x + last_y - y)

        if dist < 0.1 then
            frames_little_change = frames_little_change + 1
        else
            frames_little_change = 0
        end

        local rand_val = ProceduralRandomf(GameGetFrameNum(), (x / 2) + (y / 2))
        local base_prob = (1.0 + frames_little_change) / 500
        base_prob = base_prob > 1.0 and 1.0 or base_prob
        if rand_val < base_prob then
            last_movement_dir = (rand_val < (base_prob / 2)) and 1 or -1
        end

        local closest_player = get_closest_real_player(entity_id)
        if closest_player then
            px, py = EntityGetTransform(closest_player)
            force_magnitude = 50
        end

        if px then
            dx = x - px
            dy = y - py
            dist = math.sqrt(dx*dx + dy*dy)
        end

        if (not closest_player) or (dist > 100) or (not px) then
            px = last_movement_dir + x
            py = y
            dx = x - px
            dy = y - py
            dist = math.sqrt(dx*dx + dy*dy)
            force_magnitude = 40
        end

        if any_leg_attached then
            local fx = (dx / dist) * force_magnitude
            local fy = (dy / dist) * force_magnitude - 8
    
            PhysicsApplyForce(entity_id, fx, fy)
        end

        last_x = x
        last_y = y
        if GameGetFrameNum() % 60 == 53 then
            EntitySetTransform(entity_id, x, y, 0, math.abs(scale_x), math.abs(scale_y))
        end

    end
end

return heart_statue
