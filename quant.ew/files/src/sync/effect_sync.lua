local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local np = require("noitapatcher")

local rpc = net.new_rpc_namespace()

local effect_sync = {}

function effect_sync.get_ent_effects(entity)
    local list = {}
    for _, ent in ipairs(EntityGetAllChildren(entity) or {}) do
        local com = EntityGetFirstComponentIncludingDisabled(ent, "GameEffectComponent")
        if com ~= nil then
            -- local name = ComponentGetValue2(com, "effect")
            -- GamePrint("eff "..name)
            table.insert(list, ent)
        end
    end
    return list
end

function effect_sync.on_world_update()
    if GameGetFrameNum() % 30 ~= 9 then
        return
    end
    local all_sync_data = {}

    local effects = effect_sync.get_ent_effects(ctx.my_player.entity)
    local sync_data = {}
    for _, effect in ipairs(effects) do
        table.insert(sync_data, np.SerializeEntity(effect))
    end
    all_sync_data[ctx.my_id] = sync_data

    rpc.send_effects(all_sync_data)
end

function effect_sync.remove_all_effects(entity)
    local effects = effect_sync.get_ent_effects(entity)
    for _, effect in ipairs(effects) do
        EntityKill(effect)
    end
end

function rpc.send_effects(effects_of_players)
    for peer_id, effects in pairs(effects_of_players) do
        local entity = player_fns.peer_get_player_data(peer_id).entity
        effect_sync.remove_all_effects(entity)
        for _, effect in ipairs(effects) do
            local ent = EntityCreateNew()
            np.DeserializeEntity(ent, effect)
            EntityAddChild(entity, ent)
        end
    end
end

return effect_sync