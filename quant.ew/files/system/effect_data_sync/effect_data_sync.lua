-- This module intends to sync stains.
-- Both stains and ingestion effects are handled by StatusEffectDataComponent, thus the name.

dofile_once("data/scripts/status_effects/status_list.lua")

local effect_by_index = {}

for index, effect in ipairs(status_effects) do
    effect_by_index[index] = effect.id
end

local rpc = net.new_rpc_namespace()

local module = {}

function rpc.send_present_stains(present_stains)
    local entity = ctx.rpc_player_data.entity
    local effect_data = EntityGetFirstComponent(entity, "StatusEffectDataComponent")
    if effect_data == nil or effect_data == 0 then
        return
    end
    local current_stains = ComponentGetValue2(effect_data, "stain_effects")

    for index, is_present in ipairs(present_stains) do
        if not is_present and current_stains[index+1] ~= nil and current_stains[index+1] > 0.1 then
            --GamePrint("Removing "..effect_by_index[index])
            EntityRemoveStainStatusEffect(entity, effect_by_index[index])
        end
    end
end

function module.on_world_update()
    if GameGetFrameNum() % 30 ~= 13 then
        return
    end
    local entity = ctx.my_player.entity
    local effect_data = EntityGetFirstComponent(entity, "StatusEffectDataComponent")
    if effect_data == nil or effect_data == 0 then
        return
    end

    local stains = ComponentGetValue2(effect_data, "stain_effects")
    local present_stains = {}
    -- For some reason whatever value is at index 1 isn't used?
    for i=2, #stains do
        table.insert(present_stains, stains[i] > 0.1)
        --[[if stains[i] > 0.1 then
            GamePrint(effect_by_index[i-1])
        end]]
    end
    rpc.send_present_stains(present_stains)
end

return module