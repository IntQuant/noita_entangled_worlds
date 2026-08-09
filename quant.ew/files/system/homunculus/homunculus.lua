local rpc = net.new_rpc_namespace()
local homunculus = {}

-- EntityGetWithTag("lukki") is an engine-wide scan. It used to run once per
-- frame per player, including for players who never took the perk. Lukki perk
-- entities are created/destroyed rarely, so the owning scan is cached per
-- player entity and refreshed periodically; positions are still read every
-- frame off the cached list.
local LUKKI_RESCAN_INTERVAL = 20
local lukki_cache = {}

local function get_lukki(entity)
    local frame = GameGetFrameNum()
    local cached = lukki_cache[entity]
    if cached == nil or frame - cached.frame >= LUKKI_RESCAN_INTERVAL then
        -- Drop entries for players that no longer exist so this can't grow.
        for ent, _ in pairs(lukki_cache) do
            if not EntityGetIsAlive(ent) then
                lukki_cache[ent] = nil
            end
        end
        local found = {}
        for _, child in ipairs(EntityGetWithTag("lukki") or {}) do
            if EntityHasTag(child, "perk_entity") then
                -- EntityGetComponent returns nil (not an empty table) when the
                -- entity has no such component.
                local comps = EntityGetComponent(child, "VariableStorageComponent")
                local var = comps ~= nil and comps[2] or nil
                if var ~= nil and ComponentGetValue2(var, "value_int") == entity then
                    table.insert(found, child)
                end
            end
        end
        cached = { frame = frame, list = found }
        lukki_cache[entity] = cached
    end
    local alive = {}
    for _, child in ipairs(cached.list) do
        if EntityGetIsAlive(child) then
            table.insert(alive, child)
        end
    end
    return alive
end

local function get_entities(entity)
    local homunculy = {}
    local ghost = {}
    for _, child in ipairs(EntityGetAllChildren(entity) or {}) do
        if EntityHasTag(child, "homunculus") then
            table.insert(homunculy, child)
        elseif
            EntityHasTag(child, "angry_ghost")
            or EntityHasTag(child, "hungry_ghost")
            or EntityHasTag(child, "ghostly_ghost")
            or EntityHasTag(child, "death_ghost")
        then
            table.insert(ghost, child)
        end
    end
    return homunculy, get_lukki(entity), ghost
end
rpc.opts_reliable()
function rpc.send_positions(ho, lu, gh, f)
    local h, l, g = get_entities(ctx.rpc_player_data.entity)
    if #ho ~= 0 then
        for i, child in ipairs(h) do
            if ho[i] == nil then
                EntityKill(child)
            else
                EntitySetTransform(child, ho[i][1], ho[i][2])
            end
        end
        for i = 1, #ho - #h do
            local n = EntityLoad("data/entities/misc/homunculus.xml", ho[#h + i][1], ho[#h + i][2])
            util.make_ephemerial(n)
            EntityAddChild(ctx.rpc_player_data.entity, n)
        end
    end
    if #lu ~= 0 then
        for i, child in ipairs(l) do
            if lu[i] == nil then
                EntityKill(child)
            else
                util.set_phys_info(child, lu[i][3], ctx.rpc_player_data.fps)
            end
        end
        for i = 1, #lu - #l do
            local n = EntityLoad("data/entities/misc/perks/lukki_minion.xml", lu[#l + i][1], lu[#l + i][2])
            util.set_phys_info(n, lu[i][3], ctx.rpc_player_data.fps)
            EntityAddComponent(n, "VariableStorageComponent", {
                name = "owner_id",
                value_int = ctx.rpc_player_data.entity,
            })
            EntityRemoveComponent(n, EntityGetFirstComponent(n, "LuaComponent"))
            EntityAddTag(n, "perk_entity")
            util.make_ephemerial(n)
        end
    end
    if #gh ~= 0 then
        for i, child in ipairs(g) do
            if gh[i] == nil then
                EntityKill(child)
            else
                EntitySetTransform(child, gh[i][1], gh[i][2])
                local var
                for _, v in ipairs(EntityGetComponentIncludingDisabled(child, "VariableStorageComponent") or {}) do
                    if ComponentGetValue2(v, "name") == "ew_frame" then
                        var = v
                        break
                    end
                end
                if var == nil then
                    var = EntityAddComponent(child, "VariableStorageComponent")
                    ComponentSetValue2(var, "name", "ew_frame")
                end
                ComponentSetValue2(var, "value_int", f)
                local rnd = EntityGetFirstComponentIncludingDisabled(child, "VariableStorageComponent", "ew_ghost_rnd")
                if rnd ~= nil then
                    ComponentSetValue2(rnd, "value_float", gh[i][3])
                end
            end
        end
    end
end
function homunculus.on_world_update()
    local h, l, g = get_entities(ctx.my_player.entity)
    local ho, lu, gh = {}, {}, {}
    for _, child in ipairs(h) do
        local x, y = EntityGetTransform(child)
        table.insert(ho, { x, y })
    end
    for _, child in ipairs(l) do
        local x, y = EntityGetTransform(child)
        table.insert(lu, { x, y, util.get_phys_info(child) })
    end
    for _, child in ipairs(g) do
        local x, y = EntityGetTransform(child)
        local rnd = EntityGetFirstComponentIncludingDisabled(child, "VariableStorageComponent", "ew_ghost_rnd")
        if rnd ~= nil then
            rnd = ComponentGetValue2(rnd, "value_float")
            table.insert(gh, { x, y, rnd })
        end
    end
    if #ho ~= 0 or #lu ~= 0 or #gh ~= 0 then
        rpc.send_positions(ho, lu, gh, GameGetFrameNum())
    end
end
return homunculus
