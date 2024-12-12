local rpc = net.new_rpc_namespace()

local last_orb_count = 0

local last = 0

local module = {}

local wait_for_these

local function orbs_found_this_run()
    local wsc = EntityGetFirstComponent(GameGetWorldStateEntity(), "WorldStateComponent")
    return ComponentGetValue2(wsc, "orbs_found_thisrun")
end

local function actual_orbs_update(found_orbs)
    local found_local = orbs_found_this_run()
    for _, orb in ipairs(found_orbs) do
        if table.contains(found_local, orb) then
            goto continue
        end
        local orb_ent = EntityCreateNew()
        EntityAddComponent2(orb_ent, "ItemComponent", {
            enable_orb_hacks = true,
            auto_pickup = true,
        })
        EntityAddComponent2(orb_ent, "OrbComponent", {
            orb_id = orb,
        })
        EntityAddComponent2(orb_ent, "LuaComponent", {
            script_item_picked_up = "data/scripts/items/orb_pickup.lua",
        })
        local x, y = EntityGetTransform(ctx.my_player.entity)
        EntitySetTransform(orb_ent, x, y)
        ::continue::
    end
    -- Prevent excess rpcs.
    last_orb_count = GameGetOrbCountThisRun()
end

function rpc.update_orbs(found_orbs, to_host)
    if to_host and ctx.my_id ~= ctx.host_id then
        return
    end
    if EntityHasTag(ctx.my_player.entity, "polymorphed") then
        wait_for_these = found_orbs
        return
    end
    actual_orbs_update(found_orbs)
    local found_local = orbs_found_this_run()
    local n = EntitiesGetMaxID()
    for ent = last + 1, n do
        if EntityGetIsAlive(ent) then
            local comp = EntityGetFirstComponent(ent, "OrbComponent")
            if comp ~= nil then
                local orb = ComponentGetValue2(comp, "orb_id")
                if table.contains(found_local, orb) then
                    EntityKill(ent)
                end
            end
        end
    end
    last = n
end

function module.on_world_update()
    if GameGetFrameNum() % 15 == 0 then
        local n = EntitiesGetMaxID()
        local found_local = orbs_found_this_run()
        for ent = last + 1, n do
            if EntityGetIsAlive(ent) then
                local comp = EntityGetFirstComponent(ent, "OrbComponent")
                if comp ~= nil then
                    local orb = ComponentGetValue2(comp, "orb_id")
                    if table.contains(found_local, orb) then
                        EntityKill(ent)
                    end
                end
            end
        end
        last = n
    end
    if wait_for_these ~= nil and not EntityHasTag(ctx.my_player.entity, "polymorphed") then
        actual_orbs_update(wait_for_these)
        wait_for_these = nil
    elseif last_orb_count ~= GameGetOrbCountThisRun() then
        last_orb_count = GameGetOrbCountThisRun()
        rpc.update_orbs(orbs_found_this_run(), ctx.my_id ~= ctx.host_id)
    end
end

return module