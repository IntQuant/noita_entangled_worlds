local inventory_helper = dofile_once("mods/quant.ew/files/src/inventory_helper.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local util = dofile_once("mods/quant.ew/files/src/util.lua")

local item_sync = {}

local function mark_in_inventory(my_player)
    local items = inventory_helper.get_all_inventory_items(my_player)
    for _, ent in pairs(items) do
        -- GamePrint(tostring(ent))
        EntityAddTag(ent, "ew_was_in_inventory")
        local notify = EntityGetComponentIncludingDisabled(ent, "LuaComponent", "ew_notify_component")
        if notify == nil then
            GamePrint("Added lua component")
            EntityAddComponent2(ent, "LuaComponent", {
                _tags = "enabled_in_world,enabled_in_hand,enabled_in_inventory,ew_notify_component",
                script_throw_item = "mods/quant.ew/files/cbs/item_notify.lua",
                script_item_picked_up = "mods/quant.ew/files/cbs/item_notify.lua",
                -- script_kick = "mods/quant.ew/files/cbs/item_notify.lua",
            })
        end
    end
end

local function is_item_on_ground(item)
    return EntityGetComponent(item, "SimplePhysicsComponent") ~= nil or EntityGetComponent(item, "PhysicsBodyComponent")
end

function item_sync.make_item_global(item)
    local item_data = inventory_helper.serialize_single_item(item)
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
        -- local x, y = EntityGetTransform(my_player.entity)
        -- local ents = EntityGetInRadiusWithTag(x, y, 300, "ew_was_in_inventory")
        -- for _, ent in pairs(ents) do
        --     if is_item_on_ground(ent) then
        --         if not EntityHasTag(ent, "ew_global_item") then
        --             EntityAddTag(ent, "ew_global_item")
        --             GamePrint(tostring(ent).." "..EntityGetTags(ent))
        --             item_sync.make_item_global(ent)
        --         end
        --     end
        -- end
    end
    local thrown_item = get_global_ent("ew_thrown")
    if thrown_item ~= nil then
        EntityAddTag(thrown_item, "ew_global_item")
       item_sync.make_item_global(thrown_item)
    end
end

return item_sync