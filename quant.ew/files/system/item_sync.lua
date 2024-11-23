-- Synchronizes item pickup and item drop
local inventory_helper = dofile_once("mods/quant.ew/files/core/inventory_helper.lua")
local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")

ModLuaFileAppend("data/scripts/items/utility_box.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/items/chest_random.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/items/chest_random_super.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")

dofile_once("data/scripts/lib/coroutines.lua")

local rpc = net.new_rpc_namespace()

local item_sync = {}

local pending_remove = {}
local pickup_handlers = {}

local dead_entities = {}

local frame = {}

local gid_last_frame_updated = {}

function rpc.open_chest(gid)
    local ent = item_sync.find_by_gid(gid)
    if ent ~= nil then
        local file
        local name = EntityGetFilename(ent)
        if name == "data/entities/items/pickup/utility_box.xml" then
            file = "data/scripts/items/utility_box.lua"
        elseif name == "data/entities/items/pickup/chest_random_super.xml" then
            file = "data/scripts/items/chest_random_super.lua"
        elseif name == "data/entities/items/pickup/chest_random.xml" then
            file = "data/scripts/items/chest_random.lua"
        end
        if file ~= nil then
            EntityAddComponent2(ent, "LuaComponent", {
                script_source_file = file,
                execute_on_added = true,
                call_init_function = true,
            })
        end
    end
end

util.add_cross_call("ew_chest_opened", function(chest_id)
    local gid = item_sync.get_global_item_id(chest_id)
    if gid ~= "unknown" then
        rpc.open_chest(gid)
    end
end)

util.add_cross_call("ew_item_death_notify", function(enemy_id, responsible_id)
    local player_data = player_fns.get_player_data_by_local_entity_id(responsible_id)
    local responsible
    if player_data ~= nil then
        responsible = player_data.peer_id
    else
        responsible = responsible_id
    end
    table.insert(dead_entities, {item_sync.get_global_item_id(enemy_id), responsible})
end)

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
        if not EntityHasTag(ent, "polymorphed_player") then
            item_sync.ensure_notify_component(ent)
        end
    end
end

local function allocate_global_id()
    local current = tonumber(GlobalsGetValue("ew_global_item_id", "1"))
    GlobalsSetValue("ew_global_item_id", tostring(current + 1))
    return ctx.my_player.peer_id .. ":" .. current
end

-- Try to guess if the item is in world.
local function is_item_on_ground(item)
    return EntityGetRootEntity(item) == item
end

function item_sync.get_global_item_id(item)
    local gid = EntityGetFirstComponentIncludingDisabled(item, "VariableStorageComponent", "ew_global_item_id")
    if gid == nil then
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
    if find_by_gid_cache[gid] ~= nil then
        if EntityGetIsAlive(find_by_gid_cache[gid]) and EntityHasTag(find_by_gid_cache[gid], "ew_global_item") then
            return find_by_gid_cache[gid]
        else
            find_by_gid_cache[gid] = nil
        end
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
    local item = item_sync.find_by_gid(gid)
    if item ~= nil and is_item_on_ground(item) then
        for _, audio in ipairs(EntityGetComponent(item, "AudioComponent") or {}) do
            if string.sub(ComponentGetValue2(audio, "event_root"), 1, 10) == "collision/" then
                EntitySetComponentIsEnabled(item, audio, false)
            end
        end
        EntityKill(item)
    end
end

function item_sync.host_localize_item(gid, peer_id)
    if ctx.item_prevent_localize[gid] then
        print("Item localize for " .. gid .. " prevented")
        return
    end
    ctx.item_prevent_localize[gid] = true

    if table.contains(pending_remove, gid) then
        print("Item localize prevented, already taken")
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

local wait_for_gid = {}

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

        local hp, max_hp, has_hp = util.get_ent_health(item)
        if has_hp then
            util.ensure_component_present(item, "LuaComponent", "ew_item_death_notify", {
                script_death = "mods/quant.ew/files/resource/cbs/item_death_notify.lua"
            })
        end

        if give_authority_to ~= nil then
            local itemcom = EntityGetFirstComponentIncludingDisabled(item, "ItemComponent")
        end

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

local function is_my_item(gid)
    return string.sub(gid, 1, 16) == ctx.my_id
end

rpc.opts_reliable()
function rpc.handle_death_data(death_data)
    for _, remote_data in ipairs(death_data) do
        local remote_id = remote_data[1]
        --[[if confirmed_kills[remote_id] then
            GamePrint("Remote id has been killed already..?")
            goto continue
        end
        confirmed_kills[remote_id] = true]]
        local responsible_entity = 0
        local peer_data = player_fns.peer_get_player_data(remote_data[2], true)
        if peer_data ~= nil then
            responsible_entity = peer_data.entity
        elseif ctx.entity_by_remote_id[remote_data[2]] ~= nil then
            responsible_entity = ctx.entity_by_remote_id[remote_data[2]]
        end

        --if unsynced_enemys[remote_id] ~= nil then
            --sync_enemy(unsynced_enemys[remote_id], true)
        --end
        local enemy_id = item_sync.find_by_gid(remote_id)
        if enemy_id ~= nil and EntityGetIsAlive(enemy_id) then
            local immortal = EntityGetFirstComponentIncludingDisabled(enemy_id, "LuaComponent", "ew_immortal")
            if immortal ~= 0 then
                EntityRemoveComponent(enemy_id, immortal)
            end
            local protection_component_id = GameGetGameEffect(enemy_id, "PROTECTION_ALL")
            if protection_component_id ~= 0 then
                EntitySetComponentIsEnabled(enemy_id, protection_component_id, false)
            end

            local damage_component = EntityGetFirstComponentIncludingDisabled(enemy_id, "DamageModelComponent")
            if damage_component and damage_component ~= 0 then
                ComponentSetValue2(damage_component, "wait_for_kill_flag_on_death", false)
            end

            -- Enable explosion back
            local expl_component = EntityGetFirstComponent(enemy_id, "ExplodeOnDamageComponent")
            if expl_component ~= nil and expl_component ~= 0 then
                ComponentSetValue2(expl_component, "explode_on_death_percent", 1)
            end

            local current_hp = util.get_ent_health(enemy_id)
            local dmg = current_hp
            if dmg > 0 then
                EntityInflictDamage(enemy_id, dmg+0.1, "DAMAGE_CURSE", "", "NONE", 0, 0, responsible_entity)
            end

            EntityInflictDamage(enemy_id, 1000000000, "DAMAGE_CURSE", "", "NONE", 0, 0, responsible_entity) -- Just to be sure
            async(function()
                wait(1)
                EntityKill(enemy_id)
            end)
        end
        ::continue::
    end
end

local DISTANCE_LIMIT = 128 * 4

local function send_item_positions(all)
    local position_data = {}
    for _, item in ipairs(EntityGetWithTag("ew_global_item")) do
        local gid = item_sync.get_global_item_id(item)
        -- Only send info about items created by us.
        if is_my_item(gid) and is_item_on_ground(item) then
            local phys_info = util.get_phys_info(item, true)
            local x, y = EntityGetTransform(item)
            if (phys_info[1][1] ~= nil or phys_info[2][1] ~= nil or all)
                    and (#EntityGetInRadiusWithTag(x, y, DISTANCE_LIMIT, "ew_peer") ~= 0
                    or #EntityGetInRadiusWithTag(x, y, DISTANCE_LIMIT, "polymorphed_player") ~= 0) then
                local costcom = EntityGetFirstComponentIncludingDisabled(item, "ItemCostComponent")
                local cost = 0
                if costcom ~= nil then
                    cost = ComponentGetValue2(costcom, "cost")
                    local mx, my = GameGetCameraPos()
                    if math.abs(mx - x) < 1024 and math.abs(my - y) < 1024
                            and EntityGetFirstComponentIncludingDisabled(item, "VariableStorageComponent", "ew_try_stealable") then
                        ComponentSetValue2(costcom, "stealable", true)
                    end
                end
                position_data[gid] = {x, y, phys_info, cost}
            end
        end
    end
    rpc.update_positions(position_data, all)
    if #dead_entities > 0 then
        rpc.handle_death_data(dead_entities)
    end
    dead_entities = {}
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
    if picked_item ~= nil and EntityHasTag(picked_item, "ew_global_item")
            and EntityHasTag(EntityGetRootEntity(picked_item), "ew_peer") then
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
    if picked_item ~= nil and EntityHasTag(picked_item, "ew_global_item")
            and EntityHasTag(EntityGetRootEntity(picked_item), "ew_peer") then
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
    if GameGetFrameNum() % 60 == 35 then
        for _, ent in ipairs(EntityGetWithTag("mimic_potion")) do
            if not EntityHasTag(ent, "polymorphed_player") and is_item_on_ground(ent) then
                if not EntityHasTag(ent, "ew_global_item") then
                    if ctx.is_host then
                        item_sync.make_item_global(ent)
                    else
                        EntityKill(ent)
                    end
                end
            end
        end
    end
    if GameGetFrameNum() % 60 == 3 then
        send_item_positions(true)
    elseif GameGetFrameNum() % 5 == 3 then
        send_item_positions(false)
    end
end

function item_sync.on_should_send_updates()
    if not ctx.is_host then
        return
    end
    local global_items = EntityGetWithTag("ew_global_item")
    local item_list = {}
    for _, item in ipairs(global_items) do
        if is_item_on_ground(item) and not EntityHasTag(item, "mimic_potion") then
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
            local on_ground, reason = is_item_on_ground(ent)
            if on_ground then
                imgui.Text("On ground: "..reason)
            else
                imgui.Text("Not on ground: "..reason)
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
    wait_for_gid[item_data.gid] = nil
    if is_safe_to_remove() then
        item_sync.remove_item_with_id_now(item_data.gid)
    end
    local n = item_sync.find_by_gid(item_data.gid)
    if n ~= nil and EntityGetRootEntity(n) == n then
        return
    end
    local item = inventory_helper.deserialize_single_item(item_data)
    add_stuff_to_globalized_item(item, item_data.gid)
    local coms = EntityGetComponent(item, "VariableStorageComponent")
    for _, com in ipairs(coms) do
        if ComponentGetValue2(com, "name") == "throw_time" then
            ComponentSetValue2(com, "value_int", GameGetFrameNum())
        end
    end
    local damage_component = EntityGetFirstComponentIncludingDisabled(item, "DamageModelComponent")
    if damage_component and damage_component ~= 0 then
        ComponentSetValue2(damage_component, "wait_for_kill_flag_on_death", true)
        EntityAddComponent2(item, "LuaComponent", {_tags="ew_immortal", script_damage_about_to_be_received = "mods/quant.ew/files/resource/cbs/immortal.lua"})
    end
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

local function cleanup(peer)
    for gid, num in pairs(gid_last_frame_updated[peer]) do
        if frame[peer] > num then
            local item = item_sync.find_by_gid(gid)
            if is_item_on_ground(item) then
                EntityKill(item)
            end
        end
    end
    gid_last_frame_updated[peer] = {}
end

function rpc.update_positions(position_data, all)
    if frame[ctx.rpc_peer_id] == nil or all then
        frame[ctx.rpc_peer_id] = GameGetFrameNum()
        if gid_last_frame_updated[ctx.rpc_peer_id] == nil then
            gid_last_frame_updated[ctx.rpc_peer_id] = {}
        end
    end
    local cx, cy = GameGetCameraPos()
    for gid, el in pairs(position_data) do
        local x, y = el[1], el[2]
        if math.abs(x - cx) < DISTANCE_LIMIT and math.abs(y - cy) < DISTANCE_LIMIT then
            gid_last_frame_updated[ctx.rpc_peer_id][gid] = frame[ctx.rpc_peer_id]
            local phys_info = el[3]
            local price = el[4]
            local item = item_sync.find_by_gid(gid)
            if item ~= nil then
                if not util.set_phys_info(item, phys_info) then
                    EntitySetTransform(item, x, y)
                end
                local costcom = EntityGetFirstComponentIncludingDisabled(item, "ItemCostComponent")
                if costcom ~= nil then
                    if price == 0 then
                        EntitySetComponentsWithTagEnabled(item, "shop_cost", false)
                        ComponentSetValue2(costcom, "cost", 0)
                    else
                        EntitySetComponentsWithTagEnabled(item, "shop_cost", true)
                        ComponentSetValue2(costcom, "cost", price)
                    end
                end
            else
                util.log("Requesting again "..gid)
                if wait_for_gid[gid] == nil then
                    rpc.request_send_again(gid)
                    wait_for_gid[gid] = true
                end
            end
        end
    end
    if all then
        async(function()
            wait(1)
            cleanup(ctx.rpc_peer_id)
        end)
    end
end

function rpc.request_send_again(gid)
    if not is_my_item(gid) then
        return
    end
    local item = item_sync.find_by_gid(gid)
    if item == nil then
        util.log("Requested to send item again, but this item wasn't found: "..gid)
        return
    end
    item_sync.make_item_global(item)
end

ctx.cap.item_sync = {
    globalize = item_sync.make_item_global,
    register_pickup_handler = function(handler)
        table.insert(pickup_handlers, handler)
    end
}

item_sync.rpc = rpc

return item_sync