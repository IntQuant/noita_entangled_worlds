local rpc = net.new_rpc_namespace()

local karl = {}

local my_karl

function rpc.send_karl(x, y, vx, vy, t, jet)
    local players_karl
    for _, entity in ipairs(EntityGetWithTag("racing_cart")) do
        local com = EntityGetFirstComponentIncludingDisabled(entity, "VariableStorageComponent", "ew_karl")
        if ComponentGetValue2(com, "value_string") == ctx.rpc_peer_id then
            players_karl = entity
            break
        end
    end
    if players_karl == nil then
        players_karl = EntityLoad("data/entities/buildings/racing_cart.xml", x, y)
        EntitySetTransform(players_karl, x, y, t)
        EntityAddComponent2(players_karl, "VariableStorageComponent", {_tags = "ew_karl", value_string = ctx.rpc_peer_id})
        for _, com in ipairs(EntityGetComponent(players_karl, "LuaComponent")) do
            if ComponentGetValue2(com, "script_source_file") == "data/scripts/buildings/racing_cart_move.lua" then
                EntityRemoveComponent(players_karl, com)
                break
            end
        end
    else
        EntitySetTransform(players_karl, x, y, t)
    end
    EntitySetComponentsWithTagEnabled(players_karl, "jetpack", jet)
    local vel = EntityGetFirstComponentIncludingDisabled(players_karl, "VelocityComponent")
    ComponentSetValue2(vel, "mVelocity", vx, vy)
end

function karl.on_world_update()
    if my_karl == nil or not EntityGetIsAlive(my_karl) then
        my_karl = nil
        if GameGetFrameNum() % 30 == 0 then
            for _, entity in ipairs(EntityGetWithTag("racing_cart")) do
                local com = EntityGetFirstComponentIncludingDisabled(entity, "VariableStorageComponent", "ew_karl")
                if com == nil then
                    my_karl = entity
                    break
                end
            end
        end
    else
        local x, y, t = EntityGetTransform(my_karl)
        local vel = EntityGetFirstComponentIncludingDisabled(my_karl, "VelocityComponent")
        local vx, vy = ComponentGetValue2(vel, "mVelocity")
        local jet = ComponentGetIsEnabled(EntityGetFirstComponentIncludingDisabled(my_karl, "SpriteParticleEmitterComponent"))
        rpc.send_karl(x, y, vx, vy, t, jet)
    end
end

return karl