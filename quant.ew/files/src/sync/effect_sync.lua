local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
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
    local my_player = ctx.my_player
    local effects = effect_sync.get_ent_effects(my_player.entity)
    local sync_data = {}
    for _, effect in ipairs(effects) do
        table.insert(sync_data, np.SerializeEntity(effect))
    end
    rpc.send_effects(sync_data)
end

function effect_sync.remove_all_effects(entity)
    local effects = effect_sync.get_ent_effects(entity)
    for _, effect in ipairs(effects) do
        EntityKill(effect)
    end
end

function rpc.send_effects(effects)
    local entity = ctx.rpc_player_data.entity
    effect_sync.remove_all_effects(entity)
    for _, effect in ipairs(effects) do
        local ent = EntityCreateNew()
        np.DeserializeEntity(ent, effect)
        EntityAddChild(entity, ent)
    end
end

return effect_sync