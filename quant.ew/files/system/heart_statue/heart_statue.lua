local rpc = net.new_rpc_namespace()

local heart_statue = {}

-- Manually remove item from the inventory
local function enable_in_world(item)
    if not item or not EntityGetIsAlive(item) then
        return
    end
    EntitySetComponentsWithTagEnabled(item, "enabled_in_hand", false)
    EntitySetComponentsWithTagEnabled(item, "enabled_in_inventory", false)
    EntitySetComponentsWithTagEnabled(item, "enabled_in_world", true)
    if EntityGetParent(item) ~= 0 then
        EntityRemoveFromParent(item)
    end
end

local function find_player_cape(entity)
    if not entity or not EntityGetIsAlive(entity) then
        return nil
    end
    local cape
    local player_child_entities = EntityGetAllChildren(entity)
    if player_child_entities ~= nil then
        for _, child_entity in ipairs(player_child_entities) do
            local child_entity_name = EntityGetName(child_entity)
            if child_entity_name == "cape" then
                cape = child_entity
                break
            end
        end
    end

    return cape
end

rpc.opts_everywhere()
rpc.opts_reliable()
function rpc.add_player_cape_for_fun(peer_id)
    local player_data = ctx.players[peer_id]
    if not player_data or not player_data.entity or not EntityGetIsAlive(player_data.entity) then
        return
    end
    local entity = player_data.entity
    local cape = find_player_cape(entity)

    if cape then
        EntityRemoveFromParent(cape)
        EntityKill(cape)
    end

    local player_cape_sprite_file = "mods/quant.ew/files/system/player/tmp/" .. peer_id .. "_cape.xml"
    local x, y = EntityGetTransform(entity)
    local cape2 = EntityLoad(player_cape_sprite_file, x, y)
    if cape2 and EntityGetIsAlive(cape2) then
        EntityAddChild(entity, cape2)
    end
end

rpc.opts_everywhere()
rpc.opts_reliable()
function rpc.remove_cape(peer_id)
    local player_data = ctx.players[peer_id]
    if not player_data or not player_data.entity or not EntityGetIsAlive(player_data.entity) then
        return
    end
    local entity = player_data.entity
    local cape = find_player_cape(entity)

    if cape then
        EntityRemoveFromParent(cape)
        EntityKill(cape)
    end
end

local function remove_legs(entity)
    if not entity or not EntityGetIsAlive(entity) then
        return
    end
    local child_entities = EntityGetAllChildren(entity)
    if child_entities ~= nil then
        for _, child_entity in ipairs(child_entities) do
            local child_entity_name = EntityGetName(child_entity)
            if child_entity_name == "hs_leg" then
                EntityRemoveFromParent(child_entity)
                EntityKill(child_entity)
            end
        end
    end

    local animator = EntityGetFirstComponentIncludingDisabled(entity, "IKLimbsAnimatorComponent")
    if animator then
        EntityRemoveComponent(entity, animator)
    end
end

rpc.opts_everywhere()
rpc.opts_reliable()
function rpc.remove_legs(peer_id)
    if ctx.proxy_opt.local_health_alternate_dont_run then
        return
    end

    local player_data = ctx.players[peer_id]
    if player_data and player_data.entity and EntityGetIsAlive(player_data.entity) then
        remove_legs(player_data.entity)
    end
end

local function add_legs(entity)
    if ctx.proxy_opt.local_health_alternate_dont_run then
        return
    end

    remove_legs(entity)

    async(function()
        wait(1)
        if not entity or not EntityGetIsAlive(entity) then
            return
        end
        local x, y = EntityGetTransform(entity)
        for i = 1, 5 do
            local leg = EntityLoad("mods/quant.ew/files/system/heart_statue/heart_statue_leg.xml", x, y)
            local limb_component = EntityGetFirstComponentIncludingDisabled(leg, "IKLimbComponent")
            if limb_component then
                ComponentSetValue2(limb_component, "end_position", x, y)
                ComponentSetValue2(limb_component, "mJointWorldPos", x, y)
                ComponentSetValue2(limb_component, "mEndPrevPos", x, y)
            end
            EntityAddChild(entity, leg)
        end
    end)

    EntityAddComponent2(entity, "IKLimbsAnimatorComponent", {
        _tags = "enabled_in_world",
        ray_skip_material = CellFactory_GetType("rock_box2d_nohit_hard"),
        ground_attachment_ray_length_coeff = 0.75,
        future_state_samples = 5,
        leg_velocity_coeff = 30,
        no_ground_attachment_penalty_coeff = 0.01,
    })
end

rpc.opts_everywhere()
rpc.opts_reliable()
function rpc.got_thrown(peer_id, phys_transform)
    local player_data = ctx.players[peer_id]
    if not player_data or not player_data.entity or not EntityGetIsAlive(player_data.entity) then
        return
    end
    local item = player_data.entity
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

    local owner_data = player_fns.get_player_data_by_local_entity_id(item)
    if owner_data then
        local owner_peer_id = owner_data.peer_id
        if owner_peer_id == ctx.my_player.peer_id then
            rpc.disable_running(ctx.my_player.peer_id)
            ctx.my_player.heart_statue_running_data.running_start_frame = GameGetFrameNum() + 180
        end
    end

    if peer_id == ctx.my_player.peer_id and ctx.my_player and ctx.my_player.entity and EntityGetIsAlive(ctx.my_player.entity) then
        local phys_component = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "PhysicsBodyComponent")
        local t = phys_transform
        if phys_component and t and t.px and t.py and t.pr and t.pvx and t.pvy and t.pvr then
            np.PhysBodySetTransform(phys_component, t.px, t.py, t.pr, t.pvx, t.pvy, t.pvr)
        end
    end
end

util.add_cross_call("ew_heart_statue_throw", function(item)
    -- Transfering physic transform from the player that throws the item to other players
    local phys_component = EntityGetFirstComponentIncludingDisabled(item, "PhysicsBodyComponent")
    local px, py, pr, pvx, pvy, pvr = np.PhysBodyGetTransform(phys_component)
    local peer_id = player_fns.get_player_data_by_local_entity_id(item).peer_id
    rpc.got_thrown(
        peer_id,
        { px = px, py = py, pr = pr, pvx = pvx, pvy = pvy, pvr = pvr }
    )
    rpc.add_player_cape_for_fun(peer_id)
end)

util.add_cross_call("ew_heart_statue_pickup", function(item)
    if item then
        local peer_id = player_fns.get_player_data_by_local_entity_id(item).peer_id
        rpc.remove_cape(peer_id)
        rpc.disable_running(peer_id)

        async(function ()
            wait(10)
            rpc.remove_legs(peer_id)
        end)
    end

    local inventory_state = player_fns.serialize_items(ctx.my_player)
    if inventory_state ~= nil then
        net.send_player_inventory(inventory_state)
    end
end)

rpc.opts_reliable()
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

rpc.opts_everywhere()
rpc.opts_reliable()
function rpc.start_movement(peer_id)
    if ctx.proxy_opt.local_health_alternate_dont_run then
        return
    end

    local entity_id = ctx.players[peer_id].entity
    add_legs(entity_id)
    EntityAddTag(entity_id, "ew_hs_moving")
end

local function spawn_pedestal(x, y)
    LoadPixelScene(
        "mods/quant.ew/files/system/heart_statue/heart_statue_pedestal.png",
        "mods/quant.ew/files/system/heart_statue/heart_statue_pedestal_visual.png",
        x - 8,
        y - 9,
        "",
        true,
        true
    )
end

rpc.opts_reliable()
function rpc.spawn_pedestal(x, y)
    async(function ()
        --delay is here to hopefully avoid not having a texture
        wait(10)
        spawn_pedestal(x, y)
    end)
end

rpc.opts_everywhere()
rpc.opts_reliable()
function rpc.disable_running(peer_id)
    local entity_id = ctx.players[peer_id].entity
    EntityRemoveTag(entity_id, "ew_hs_moving")
end

util.add_cross_call("ew_heart_statue_spawn_pedestal", function(entity_id)
    local x, y = EntityGetTransform(entity_id)
    local owner_peer_id = player_fns.get_player_data_by_local_entity_id(entity_id).peer_id
    if ctx.my_player.peer_id == ctx.host_id then
        spawn_pedestal(x, y)
        rpc.spawn_pedestal(x, y)
    end

    if owner_peer_id == ctx.my_player.peer_id then
        rpc.disable_running(owner_peer_id)
        rpc.add_player_cape_for_fun(owner_peer_id)
        if not ctx.my_player.heart_statue_running_data then
            ctx.my_player.heart_statue_running_data = {
                last_movement_dir = 1,
                last_x = 0,
                last_y = 0,
                dist_accum = 0,
                frames_little_change = 0,
                running_start_frame = GameGetFrameNum() + 180
            }
        end
    end
end)

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

local function run_away_from_player(data, entity_id)
    local dist_accum = data.dist_accum
    local frames_little_change = data.frames_little_change

    local ik_animator = EntityGetFirstComponent(entity_id, "IKLimbsAnimatorComponent")
    local any_leg_attached = false
    if ik_animator ~= nil then
        any_leg_attached = ComponentGetValue2(ik_animator, "mHasGroundAttachmentOnAnyLeg")
    end

    local velocity_comp = EntityGetFirstComponent(entity_id, "VelocityComponent")
    local vel_x, vel_y = 0, 0
    if velocity_comp ~= nil then
        vel_x, vel_y = ComponentGetValue2(velocity_comp, "mVelocity")
    end

    local x, y, rot = EntityGetTransform(entity_id)
    local force_magnitude_x, force_magnitude_y = 0, 0
    local px, py
    local dx, dy
    local prob_mult = 1.0
    local dist = math.abs((data.last_x - x) + (data.last_y - y))
    local max_vel_diff_x = math.max(0, 75 - math.abs(vel_x))
    local max_vel_diff_y = math.max(0, 75 - math.abs(vel_y))

    if dist < 0.25 then
        dist_accum = dist_accum + dist
        frames_little_change = frames_little_change + 1
    else
        frames_little_change = 0
        dist_accum = 0
    end

    if frames_little_change > 0 then
        local avg = dist_accum / frames_little_change
        prob_mult = avg < 0.1 and frames_little_change or 0
    end

    data.dist_accum = dist_accum
    data.frames_little_change = frames_little_change

    local rand_val = ProceduralRandomf(GameGetFrameNum(), (x / 2) + (y / 2))
    local base_prob = (1.0 * prob_mult) / 500
    base_prob = base_prob > 1.0 and 1.0 or base_prob
    if rand_val < base_prob then
        data.last_movement_dir = (rand_val < (base_prob / 2)) and 1 or -1
    end

    local closest_player = get_closest_real_player(entity_id)
    if closest_player then
        px, py = EntityGetTransform(closest_player)
        force_magnitude_x = max_vel_diff_x * 1.5
        force_magnitude_y = max_vel_diff_y * 1.5
    end

    if px then
        dx = x - px
        dy = y - py
        dist = math.sqrt(dx*dx + dy*dy)
        dy = dy / 4
    end

    if (not closest_player) or (dist > 100) or (not px) then
        px = data.last_movement_dir + x
        py = y
        dx = x - px
        dy = y - py
        dist = math.sqrt(dx*dx + dy*dy)
        force_magnitude_x = max_vel_diff_x
        force_magnitude_y = max_vel_diff_y
    end

    if not any_leg_attached then
        PhysicsApplyForce(entity_id, 0, 15)
        force_magnitude_x = force_magnitude_x * 0.15
        force_magnitude_y = force_magnitude_y * 0.15
    end
    local fx = (dx / dist) * force_magnitude_x
    local fy = (dy / dist) * force_magnitude_y
    fy = (fy < 4) and (fy - 11) or fy
    PhysicsApplyForce(entity_id, fx, fy)
    PhysicsApplyTorque(entity_id, (rot) * -10)

    data.last_x = x
    data.last_y = y
end

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

    local data = player_fns.get_player_data_by_local_entity_id(root)
    local running_data = ctx.my_player.heart_statue_running_data

    if root ~= entity_id then
        if GameGetFrameNum() % 60 == 53 then
            rpc.remove_cape(ctx.my_player.peer_id)
            rpc.disable_running(ctx.my_player.peer_id)

            if data ~= nil then
                rpc.ensure_held(data.peer_id)
            end
        end
    elseif running_data and EntityHasTag(entity_id, "ew_hs_moving") then
        run_away_from_player(running_data, entity_id)
    elseif running_data and GameGetFrameNum() >= running_data.running_start_frame then
        rpc.start_movement(ctx.my_player.peer_id)
    end
end

return heart_statue
