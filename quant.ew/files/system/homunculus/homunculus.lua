local rpc = net.new_rpc_namespace()
local homunculus = {}
local function get_entities(entity)
    local homunculy = {}
    for _, child in ipairs(EntityGetAllChildren(entity) or {}) do
        if EntityHasTag(child, "homunculus") then
            table.insert(homunculy, child)
        end
    end
    local luuki = {}
    for _, child in ipairs(EntityGetWithTag("lukki") or {}) do
        if EntityHasTag(child, "perk_entity") then
            local var = EntityGetComponent(child, "VariableStorageComponent")[2]
            if var ~= nil then
                if ComponentGetValue2(var, "value_int") == entity then
                    table.insert(luuki, child)
                end
            end
        end
    end
    return homunculy, luuki
end
rpc.opts_reliable()
function rpc.send_positions(ho, lu)
    local h, l = get_entities(ctx.rpc_player_data.entity)
    if #ho ~= 0 then
        for i, child in ipairs(h) do
            EntitySetTransform(child, ho[i][1], ho[i][2])
        end
        for i=1,#ho-#h do
            local n = EntityLoad( "data/entities/misc/homunculus.xml", ho[#h+i][1], ho[#h+i][2])
            util.make_ephemerial(n)
            EntityAddChild(ctx.rpc_player_data.entity, n)
        end
    end
    if #lu ~= 0 then
        for i, child in ipairs(l) do
            util.set_phys_info(child, lu[i][3], ctx.rpc_player_data.fps)
        end
        for i=1,#lu-#l do
            local n = EntityLoad( "data/entities/misc/perks/lukki_minion.xml", lu[#l+i][1], lu[#l+i][2])
            util.set_phys_info(n, lu[i][3], ctx.rpc_player_data.fps)
            EntityAddComponent( n, "VariableStorageComponent",
                    {
                        name = "owner_id",
                        value_int = ctx.rpc_player_data.entity,
                    })
            EntityRemoveComponent(n, EntityGetFirstComponent(n, "LuaComponent"))
            EntityAddTag(n, "perk_entity")
            util.make_ephemerial(n)
        end
    end
end
function homunculus.on_world_update()
    local h, l = get_entities(ctx.my_player.entity)
    local ho, lu = {}, {}
    for _, child in ipairs(h) do
        local x, y = EntityGetTransform(child)
        table.insert(ho, {x, y})
    end
    for _, child in ipairs(l) do
        local x, y = EntityGetTransform(child)
        table.insert(lu, {x, y, util.get_phys_info(child)})
    end
    if #ho ~= 0 or #lu ~= 0 then
        rpc.send_positions(ho, lu)
    end
end
return homunculus