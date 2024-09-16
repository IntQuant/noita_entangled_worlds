-- Synchronizes item pickup and item drop
local inventory_helper = dofile_once("mods/quant.ew/files/core/inventory_helper.lua")
local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")

dofile_once("data/scripts/lib/coroutines.lua")

local rpc = net.new_rpc_namespace()

local item_sync = {}

local pending_remove = {}
local pickup_handlers = {}

function item_sync.ensure_notify_component(ent)
    local notify = EntityGetFirstComponentIncludingDisabled(ent, "LuaComponent", "ew_notify_component")
    if notify == nil then
        EntityAddComponent2(ent, "LuaComponent", {
            _tags = "enabled_in_world,enabled_in_hand,enabled_in_inventory,ew_notify_component,ew_remove_on_send",
            script_throw_item = "mods/quant.ew/files/resource/cbs/item_notify.lua",
            script_item_picked_up = "mods/quant.ew/files/resource/cbs/item_notify.lua",
            -- script_kick = "mods/quant.ew/files/resource/cbs/item_notify.lua",
        })
    end
end

local function mark_in_inventory(my_player)
    local items = inventory_helper.get_all_inventory_items(my_player)
    for _, ent in pairs(items) do
        item_sync.ensure_notify_component(ent)
    end
end

local function allocate_global_id()
    local current = tonumber(GlobalsGetValue("ew_global_item_id", "1"))
    GlobalsSetValue("ew_global_item_id", tostring(current + 1))
    return ctx.my_player.peer_id .. ":" .. current
end

-- Try to guess if the item is in world.
local function is_item_on_ground(item)
    return EntityGetComponent(item, "SimplePhysicsComponent") ~= nil or
        EntityGetComponent(item, "PhysicsBodyComponent") ~= nil or
        EntityGetComponent(item, "SpriteParticleEmitterComponent") ~= nil
end

function item_sync.get_global_item_id(item)
    local gid = EntityGetFirstComponentIncludingDisabled(item, "VariableStorageComponent", "ew_global_item_id")
    if gid == nil then
        GamePrint("Item has no gid")
        return "unknown"
    end
    local ret = ComponentGetValue2(gid, "value_string")
    return ret or "unknown"
end

function item_sync.remove_item_with_id(gid)
    table.insert(pending_remove, gid)
end

local find_by_gid_cache = {}
function item_sync.find_by_gid(gid)
    if find_by_gid_cache[gid] ~= nil and EntityGetIsAlive(find_by_gid_cache[gid]) then
        return find_by_gid_cache[gid]
    end

    --print("find_by_gid: searching")

    local global_items = EntityGetWithTag("ew_global_item")
    for _, item in ipairs(global_items) do
        local i_gid = item_sync.get_global_item_id(item)
        find_by_gid_cache[i_gid] = item
        if i_gid == gid then
            return item
        end
    end
end

function item_sync.remove_item_with_id_now(gid)
    local global_items = EntityGetWithTag("ew_global_item")
    for _, item in ipairs(global_items) do
        local i_gid = item_sync.get_global_item_id(item)
        if i_gid == gid then
            EntityKill(item)
        end
    end
end

function item_sync.host_localize_item(gid, peer_id)
    if ctx.item_prevent_localize[gid] then
        GamePrint("Item localize for " .. gid .. " prevented")
        return
    end
    ctx.item_prevent_localize[gid] = true

    if table.contains(pending_remove, gid) then
        GamePrint("Item localize prevented, already taken")
        return
    end

    local item_ent_id = item_sync.find_by_gid(gid)
    if item_ent_id ~= nil then
        for _, handler in ipairs(pickup_handlers) do
            handler(item_ent_id)
        end
    end
    if peer_id ~= ctx.my_id then
        item_sync.remove_item_with_id(gid)
    end
    rpc.item_localize(peer_id, gid)
end

function item_sync.make_item_global(item, instant, give_authority_to)
    EntityAddTag(item, "ew_global_item")
    async(function()
        if not instant then
            wait(1) -- Wait 1 frame so that game sets proper velocity.
        end
        if not EntityGetIsAlive(item) then
            print("Thrown item vanished before we could send it")
            return
        end
        item_sync.ensure_notify_component(item)
        local gid_component = EntityGetFirstComponentIncludingDisabled(item, "VariableStorageComponent",
            "ew_global_item_id")
        local gid
        if gid_component == nil then
            gid = allocate_global_id()
            if give_authority_to ~= nil then
                gid = give_authority_to..":"..gid
            end
            EntityAddComponent2(item, "VariableStorageComponent", {
                _tags = "enabled_in_world,enabled_in_hand,enabled_in_inventory,ew_global_item_id",
                value_string = gid,
            })
        else
            gid = ComponentGetValue2(gid_component, "value_string")
        end
        --local vel = EntityGetFirstComponentIncludingDisabled(item, "VelocityComponent")
        --if vel then
        --    local vx, vy = ComponentGetValue2(vel, "mVelocity")
        --end
        local item_data = inventory_helper.serialize_single_item(item)
        item_data.gid = gid
        ctx.item_prevent_localize[gid] = false
        rpc.item_globalize(item_data)
    end)
end

local function get_global_ent(key)
    local val = tonumber(GlobalsGetValue(key, "0"))
    GlobalsSetValue(key, "0")
    if val ~= 0 then
        return val
    end
end

local function remove_client_items_from_world()
    if GameGetFrameNum() % 5 ~= 3 then
        return
    end
    for _, item in ipairs(EntityGetWithTag("ew_client_item")) do
        if is_item_on_ground(item) then
            EntityKill(item)
        end
    end
end

local function send_item_positions()
    local position_data = {}
    for _, item in ipairs(EntityGetWithTag("ew_global_item")) do
        local gid = item_sync.get_global_item_id(item)
        -- Only send info about items created by us.
        if string.sub(gid, 1, 16) == ctx.my_id then
            local x, y = EntityGetTransform(item)
            position_data[gid] = {x, y}
        end
    end
    rpc.update_positions(position_data)
end

function item_sync.on_world_update_host()
    local my_player = ctx.my_player
    if GameGetFrameNum() % 5 == 4 then
        mark_in_inventory(my_player)
    end
    local thrown_item = get_global_ent("ew_thrown")
    if thrown_item ~= nil then
        item_sync.make_item_global(thrown_item)
    end
    local picked_item = get_global_ent("ew_picked")
    if picked_item ~= nil and EntityHasTag(picked_item, "ew_global_item") then
        local gid = item_sync.get_global_item_id(picked_item)
        item_sync.host_localize_item(gid, ctx.my_id)
    end
    remove_client_items_from_world()
end

function item_sync.on_world_update_client()
    local my_player = ctx.my_player
    if GameGetFrameNum() % 5 == 4 then
        mark_in_inventory(my_player)
    end
    local thrown_item = get_global_ent("ew_thrown")
    if thrown_item ~= nil and not EntityHasTag(thrown_item, "ew_client_item") then
        item_sync.make_item_global(thrown_item)
    end

    local picked_item = get_global_ent("ew_picked")
    if picked_item ~= nil and EntityHasTag(picked_item, "ew_global_item") then
        local gid = item_sync.get_global_item_id(picked_item)
        rpc.item_localize_req(gid)
    end
    remove_client_items_from_world()
end

local function is_safe_to_remove()
    return not ctx.is_wand_pickup
end

function item_sync.on_world_update()
    -- TODO check that we not removing item we are going to pick now, instead of checking if picker gui is open.
    if is_safe_to_remove() then
        if #pending_remove > 0 then
            local gid = table.remove(pending_remove)
            item_sync.remove_item_with_id_now(gid)
        end
    end
    if GameGetFrameNum() % (60*5) == 31 then
        send_item_positions()
    end
end

function item_sync.on_should_send_updates()
    if not ctx.is_host then
        return
    end
    local global_items = EntityGetWithTag("ew_global_item")
    local item_list = {}
    for _, item in ipairs(global_items) do
        if is_item_on_ground(item) then
            local item_data = inventory_helper.serialize_single_item(item)
            local gid = item_sync.get_global_item_id(item)
            item_data.gid = gid
            table.insert(item_list, item_data)
        end
    end
    rpc.initial_items(item_list)
end

function item_sync.on_draw_debug_window(imgui)
    local mx, my = DEBUG_GetMouseWorld()
    local ent = EntityGetClosestWithTag(mx, my, "ew_global_item")
    if ent ~= nil and ent ~= 0 then
        if imgui.CollapsingHeader("Item gid") then
            local x, y = EntityGetTransform(ent)
            GameCreateSpriteForXFrames("mods/quant.ew/files/resource/debug/marker.png", x, y, true, 0, 0, 1, true)
            local gid = item_sync.get_global_item_id(ent)
            imgui.Text("GID: " .. tostring(gid))
            local prevented = ctx.item_prevent_localize[gid]
            if prevented then
                imgui.Text("Localize prevented")
            else
                imgui.Text("Localize allowed")
            end
        end
    end
end

local function add_stuff_to_globalized_item(item, gid)
    EntityAddTag(item, "ew_global_item")
    item_sync.ensure_notify_component(item)
    -- GamePrint("Got global item: "..item)
    local gid_c = EntityGetFirstComponentIncludingDisabled(item, "VariableStorageComponent", "ew_global_item_id")
    if gid_c == nil then
        EntityAddComponent2(item, "VariableStorageComponent", {
            _tags = "ew_global_item_id",
            value_string = gid
        })
    else
        ComponentSetValue2(gid_c, "value_string", gid)
    end
    ctx.item_prevent_localize[gid] = false
end

local function maybe_disable_physics(item_new)
    local x, y = EntityGetTransform(item_new)
    -- Disable physics if world doesn't exist yet, so the item doesn't fall off.
    if not DoesWorldExistAt(x - 5, y - 5, x + 5, y + 5) then
        async(function()
            wait(1)
            local simple_physics_component = EntityGetFirstComponent(item_new, "SimplePhysicsComponent")
            if simple_physics_component ~= nil and simple_physics_component ~= 0 then
                EntitySetComponentIsEnabled(item_new, simple_physics_component, false)
            end
            local velocity = EntityGetFirstComponent(item_new, "VelocityComponent")
            if velocity ~= nil and velocity ~= 0 then
                EntitySetComponentIsEnabled(item_new, velocity, false)
            end
        end)
    end
end

rpc.opts_reliable()
function rpc.initial_items(item_list)
    -- Only run once ever, as it tends to duplicate items otherwise
    if GameHasFlagRun("ew_initial_items") then
        return
    end
    GameAddFlagRun("ew_initial_items")
    for _, item_data in ipairs(item_list) do
        local item = item_sync.find_by_gid(item_data.gid)
        if item == nil then
            local item_new = inventory_helper.deserialize_single_item(item_data)
            add_stuff_to_globalized_item(item_new, item_data.gid)
        end
    end
end

rpc.opts_reliable()
function rpc.item_globalize(item_data)
    if is_safe_to_remove() then
        item_sync.remove_item_with_id_now(item_data.gid)
    end
    local item = inventory_helper.deserialize_single_item(item_data)
    add_stuff_to_globalized_item(item, item_data.gid)
end

rpc.opts_reliable()
function rpc.item_localize(l_peer_id, item_id)
    local item_ent_id = item_sync.find_by_gid(item_id)
    if item_ent_id ~= nil then
        for _, handler in ipairs(pickup_handlers) do
            handler(item_ent_id)
        end
    end
    if l_peer_id ~= ctx.my_id then
        item_sync.remove_item_with_id(item_id)
    end
end

rpc.opts_reliable()
function rpc.item_localize_req(gid)
    if not ctx.is_host then
        return
    end
    item_sync.host_localize_item(gid, ctx.rpc_peer_id)
end

function rpc.update_positions(position_data)
    for gid, el in pairs(position_data) do
        local item = item_sync.find_by_gid(gid)
        if item ~= nil then
            EntitySetTransform(item, el[1], el[2])
        end
    end
end

ctx.cap.item_sync = {
    globalize = item_sync.make_item_global,
    register_pickup_handler = function(handler)
        table.insert(pickup_handlers, handler)
    end
}

return item_sync