local essence_list = {
    "fire", "laser", "air", "water", "alcohol",
}
local peer_essence_data = {}

local rpc = net.new_rpc_namespace()
local module = {}

local function essences_on_current_player_hash()
    local ret = 0
    for _, id in ipairs(essence_list) do
        if GameHasFlagRun("essence_"..id) then
            ret = ret * 2 + 1
        else
            ret = ret * 2
        end
    end
    return ret
end

local function essences_on_current_player()
    local ret = {}
    for _, id in ipairs(essence_list) do
        table.insert(ret, GameHasFlagRun("essence_"..id))
    end
    return ret
end

local last_essence_hash = 0

function module.on_world_update()
    if GameGetFrameNum() % 60 == 14 then
        local essences = essences_on_current_player_hash()
        if last_essence_hash ~= essences then
            rpc.send_essences(essences_on_current_player())
            last_essence_hash = essences
        end
    end
end

function module.on_should_send_updates()
    rpc.send_essences(essences_on_current_player())
end

local function update_essences_on(player_data)
    if not EntityGetIsAlive(player_data.entity) then
        return
    end
    local essence_effects = EntityGetAllChildren(player_data.entity, "essence_effect") or {}
    for _, essence in ipairs(essence_effects) do
        EntityRemoveFromParent(essence)
        EntityKill(essence)
    end
    for i, present in ipairs(peer_essence_data[player_data.peer_id] or {}) do
        local id = essence_list[i]
        if present then
            local ent = EntityLoad("data/entities/misc/essences/"..id..".xml")
            EntityAddChild(player_data.entity, ent)
        end
    end
end

function rpc.send_essences(essences)
    peer_essence_data[ctx.rpc_peer_id] = essences
    update_essences_on(ctx.rpc_player_data)
end

function module.on_client_spawned(_, player_data)
    update_essences_on(player_data)
end

return module