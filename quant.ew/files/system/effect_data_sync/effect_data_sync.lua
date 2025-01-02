-- This module intends to sync stains.
-- Both stains and ingestion effects are handled by StatusEffectDataComponent, thus the name.

dofile_once("data/scripts/status_effects/status_list.lua")

local effect_by_index = {}

for index, effect in ipairs(status_effects) do
    effect_by_index[index] = effect.id
end

local rpc = net.new_rpc_namespace()

local module = {}

function module.get_stains(entity)
    local effect_data = EntityGetFirstComponent(entity, "StatusEffectDataComponent")
    if effect_data == nil or effect_data == 0 then
        return
    end

    local stains = ComponentGetValue2(effect_data, "stain_effects")
    local present_stains = {}
    -- For some reason whatever value is at index 1 isn't used?
    for i = 2, #stains do
        if stains[i] >= 0.15 then
            present_stains[i] = true
        end
    end
    return present_stains
end

function module.sync_stains(present_stains, entity)
    local effect_data = EntityGetFirstComponent(entity, "StatusEffectDataComponent")
    if effect_data == nil or effect_data == 0 then
        return
    end
    local current_stains = ComponentGetValue2(effect_data, "stain_effects")

    for i = 2, #current_stains do
        if current_stains[i] >= 0.15 and not present_stains[i] then
            EntityRemoveStainStatusEffect(entity, effect_by_index[i - 1])
        end
    end
end

function rpc.send_present_stains(present_stains)
    module.sync_stains(present_stains, ctx.rpc_player_data.entity)
end

function module.on_world_update()
    if GameGetFrameNum() % 15 ~= 8 then
        return
    end
    local present = module.get_stains(ctx.my_player.entity)
    if present ~= nil then
        rpc.send_present_stains(present)
    end
end

return module
