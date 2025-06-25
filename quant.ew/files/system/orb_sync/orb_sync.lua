local rpc = net.new_rpc_namespace()

local last_orb_count = 0

local module = {}

local wait_for_these

local function orbs_found_this_run()
    local wsc = EntityGetFirstComponent(GameGetWorldStateEntity(), "WorldStateComponent")
    return ComponentGetValue2(wsc, "orbs_found_thisrun")
end

local spawned_orbs = {}

local function actual_orbs_update(found_orbs)
    local found_local = orbs_found_this_run()
    for _, orb in ipairs(found_orbs) do
        if table.contains(found_local, orb) or table.contains(spawned_orbs, orb) then
            goto continue
        end
        table.insert(spawned_orbs, orb)
        local orb_ent = EntityCreateNew()
        EntityAddTag(orb_ent, "ew_no_enemy_sync")
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

rpc.opts_reliable()
function rpc.update_orbs(found_orbs)
    if ctx.rpc_peer_id ~= ctx.host_id and not ctx.is_host then
        return
    end
    if EntityHasTag(ctx.my_player.entity, "polymorphed") then
        wait_for_these = found_orbs
        return
    end
    local found_local = orbs_found_this_run()
    for _, orb_ent in ipairs(EntityGetWithTag("hittable") or {}) do
        local comp = EntityGetFirstComponent(orb_ent, "OrbComponent")
        if comp ~= nil then
            local orb = ComponentGetValue2(comp, "orb_id")
            if table.contains(found_local, orb) or table.contains(found_orbs, orb) then
                EntityKill(orb_ent)
            end
        end
    end
    actual_orbs_update(found_orbs)
end

function module.on_new_entity(arr)
    for _, ent in ipairs(arr) do
        local comp = EntityGetFirstComponent(ent, "OrbComponent")
        if comp ~= nil then
            local found_local = orbs_found_this_run()
            local orb = ComponentGetValue2(comp, "orb_id")
            if table.contains(found_local, orb) then
                EntityKill(ent)
            end
        elseif EntityGetFilename(ent) == "data/entities/base_item.xml" then
            EntityKill(ent)
        end
        local com = EntityGetFirstComponentIncludingDisabled(ent, "AbilityComponent")
        if com ~= nil and ComponentGetValue2(com, "use_gun_script") then
            com = EntityGetFirstComponentIncludingDisabled(ent, "ItemComponent")
            if com ~= nil then
                ComponentSetValue2(com, "item_pickup_radius", 256)
            end
        end
    end
end

function module.on_world_update()
    if wait_for_these ~= nil and not EntityHasTag(ctx.my_player.entity, "polymorphed") then
        actual_orbs_update(wait_for_these)
        wait_for_these = nil
    elseif last_orb_count ~= GameGetOrbCountThisRun() or GameGetFrameNum() % (60 * 5) == 23 then
        last_orb_count = GameGetOrbCountThisRun()
        rpc.update_orbs(orbs_found_this_run())
    end
end

return module
