local rpc = net.new_rpc_namespace()

local last_orb_count = 0

local module = {}

local function orbs_found_this_run()
    local wsc = EntityGetFirstComponent(GameGetWorldStateEntity(), "WorldStateComponent")
    return ComponentGetValue2(wsc, "orbs_found_thisrun")
end

function rpc.update_orbs(found_orbs)
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
        local x, y = EntityGetTransform(ctx.my_player.entity)
        EntitySetTransform(orb_ent, x, y)
        ::continue::
    end
    -- Prevent excess rpcs.
    last_orb_count = GameGetOrbCountThisRun()
end

local function send_orbs()
    rpc.update_orbs(orbs_found_this_run())
end

function module.on_world_update()
    if last_orb_count ~= GameGetOrbCountThisRun() then
        last_orb_count = GameGetOrbCountThisRun()
        send_orbs()
    end
end

function module.on_should_send_updates()
    if ctx.is_host then
        send_orbs()
    end
end

return module
