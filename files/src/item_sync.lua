local inventory_helper = dofile_once("mods/quant.ew/files/src/inventory_helper.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local util = dofile_once("mods/quant.ew/files/src/util.lua")

local item_sync = {}

function item_sync.ensure_notify_component(ent)
    local notify = EntityGetFirstComponentIncludingDisabled(ent, "LuaComponent", "ew_notify_component")
        if notify == nil then
            -- GamePrint("Added lua component")
            EntityAddComponent2(ent, "LuaComponent", {
                _tags = "enabled_in_world,enabled_in_hand,enabled_in_inventory,ew_notify_component",
                script_throw_item = "mods/quant.ew/files/cbs/item_notify.lua",
                script_item_picked_up = "mods/quant.ew/files/cbs/item_notify.lua",
                -- script_kick = "mods/quant.ew/files/cbs/item_notify.lua",
            })
        end
end

local function mark_in_inventory(my_player)
    local items = inventory_helper.get_all_inventory_items(my_player)
    for _, ent in pairs(items) do
        -- GamePrint(tostring(ent))
        -- EntityAddTag(ent, "ew_was_in_inventory")
        item_sync.ensure_notify_component(ent)
    end
end

local function allocate_global_id()
    local current = tonumber(GlobalsGetValue("ew_global_item_id", "1"))
    GlobalsSetValue("ew_global_item_id", tostring(current+1))
    return current
end

local function is_item_on_ground(item)
    return EntityGetComponent(item, "SimplePhysicsComponent") ~= nil or EntityGetComponent(item, "PhysicsBodyComponent")
end

function item_sync.get_global_item_id(item)
    local g_id = EntityGetFirstComponentIncludingDisabled(item, "VariableStorageComponent", "ew_global_item_id")
    if g_id == nil then
        GamePrint("Item has no g_id")
        return 0
    end
    local ret = ComponentGetValue2(g_id, "value_int")
    return ret or 0
end

function item_sync.remove_item_with_id(g_id)
    -- GamePrint("Localize, Remove "..g_id)
    local global_items = EntityGetWithTag("ew_global_item")
    for _, item in ipairs(global_items) do
        local i_g_id = item_sync.get_global_item_id(item)
        -- GamePrint("Chk "..item.." g_id "..i_g_id)
        if i_g_id == g_id then
            EntityKill(item)
        end
    end
end

function item_sync.host_localize_item(gid, peer_id)
    -- GamePrint("Localize "..g_id)
    if peer_id ~= ctx.my_id then
        item_sync.remove_item_with_id(gid)
    end
    ctx.lib.net.send_localize(peer_id, gid)
end

function item_sync.make_item_global(item)
    local g_id = EntityGetFirstComponentIncludingDisabled(item, "VariableStorageComponent", "ew_global_item_id")
    local id = ComponentGetValue2(g_id, "value_int")
    if g_id == nil then
        id = allocate_global_id()
        GamePrint("Allocating id "..id.." for "..item)
        EntityAddComponent2(item, "VariableStorageComponent", {
            _tags = "enabled_in_world,enabled_in_hand,enabled_in_inventory,ew_global_item_id",
            value_int = id,
        })
    end
    GamePrint(item_sync.get_global_item_id(item))
    local item_data = inventory_helper.serialize_single_item(item)
    item_data.g_id = id
    ctx.lib.net.send_make_global(item_data)
end

local function get_global_ent(key)
    local val = tonumber(GlobalsGetValue(key, "0"))
    GlobalsSetValue(key, "0")
    if val ~= 0 then
        return val
    end
end

function item_sync.host_upload_items(my_player)
    if GameGetFrameNum() % 5 == 4 then
        mark_in_inventory(my_player)
    end
    local thrown_item = get_global_ent("ew_thrown")
    if thrown_item ~= nil then
        EntityAddTag(thrown_item, "ew_global_item")
       item_sync.make_item_global(thrown_item)
    end
    local picked_item = get_global_ent("ew_picked")
    if picked_item ~= nil and EntityHasTag(picked_item, "ew_global_item") then
        GamePrint("Picked up "..picked_item)
        local g_id = item_sync.get_global_item_id(picked_item)
        item_sync.host_localize_item(g_id, ctx.my_id)
    end
end

function item_sync.client_tick(my_player)
    local picked_item = get_global_ent("ew_picked")
    if picked_item ~= nil and EntityHasTag(picked_item, "ew_global_item") then
        GamePrint("Picked up "..picked_item)
        local gid = item_sync.get_global_item_id(picked_item)
        ctx.lib.net.send_localize_request(gid)
    end
end

return item_sync