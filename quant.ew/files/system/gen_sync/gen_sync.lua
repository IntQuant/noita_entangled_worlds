
ModLuaFileAppend("data/scripts/items/generate_shop_item.lua", "mods/quant.ew/files/system/gen_sync/append/shop_spawn.lua")

dofile("data/scripts/items/generate_shop_item.lua")

local rpc = net.new_rpc_namespace()

local module = {}

local function_registry = {
    generate_shop_item = {
        kind = "item",
        fn = ew_orig_generate_shop_item,
    },
    generate_shop_wand = {
        kind = "item",
        fn = ew_orig_generate_shop_wand,
    },
}

-- Runs a spawn fn if it wasn't called for these coordinates yet.
local function run_spawn_fn(fn_name, x, y, ...)
    -- Check if we have been called already.
    -- TODO: it's probably a bad idea to use run flags for that.
    local flag = "spwn_"..fn_name.."_"..x.."_"..y
    if GameHasFlagRun(flag) then
        return
    end
    GameAddFlagRun(flag)
    local fn_info = function_registry[fn_name]
    if fn_info == nil then
        print("Could find spawn_fn:", fn_name)
    end

    --Call the function.
    local ret = fn_info.fn(x, y, ...)

    -- Check what it returns, we might need to sync it to other clients.
    if ret == nil then
        return
    end
    -- Function returns item's entity id.
    if fn_info.kind == "item" then
        local eid = ret
        ctx.cap.item_sync.globalize(eid, true, ctx.rpc_peer_id)
        -- Avoid item losing it's cost on host.
        -- inventory_helper.make_item_stealable_later(eid)
        if ctx.rpc_peer_id ~= nil and ctx.rpc_peer_id ~= ctx.my_id then
            local item_cost_component = EntityGetFirstComponentIncludingDisabled(eid, "ItemCostComponent")
            ComponentSetValue2(item_cost_component, "stealable", false)
        else
            local mx, my = GameGetCameraPos()
            if math.abs(mx - x) > 1024 or math.abs(my - y) > 1024 then
                EntityAddComponent2(eid, "VariableStorageComponent", {_tags = "ew_try_stealable"})
                local item_cost_component = EntityGetFirstComponentIncludingDisabled(eid, "ItemCostComponent")
                ComponentSetValue2(item_cost_component, "stealable", false)
            end
        end
    end
end

rpc.opts_reliable()
function rpc.remote_run_spawn_fn(fn_name, x, y, ...)
    if not ctx.is_host then
        return
    end
    run_spawn_fn(fn_name, x, y, ...)
end

np.CrossCallAdd("ew_sync_gen", function(fn_name, x, y, ...)
    if ctx.is_host then
        run_spawn_fn(fn_name, x, y, ...)
    else
        rpc.remote_run_spawn_fn(fn_name, x, y, ...)
    end
end)

return module