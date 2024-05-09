local bitser = dofile_once("mods/quant.ew/files/lib/bitser.lua")

local util = {}
function util.get_ent_variable(entity, key)
    local storage = EntityGetFirstComponentIncludingDisabled(entity, "VariableStorageComponent", key)
    local value = ComponentGetValue2(storage, "value_string")
    if value == "" then
        return nil
    end
    return bitser.loads(value)
end

function util.set_ent_variable(entity, key, value)
    local storage = EntityGetFirstComponentIncludingDisabled(entity, "VariableStorageComponent", key)
    ComponentSetValue2(storage, "value_string", bitser.dumps(value))
end

function util.get_ent_health(entity)
    local damage_model = EntityGetFirstComponentIncludingDisabled(entity, "DamageModelComponent")
    if damage_model == nil then
        return 0, 0
    end
    local hp = ComponentGetValue2(damage_model, "hp")
    local max_hp = ComponentGetValue2(damage_model, "max_hp")
    return hp, max_hp
end

function util.set_ent_health(entity, hp_data)
    local damage_model = EntityGetFirstComponentIncludingDisabled(entity, "DamageModelComponent")
    if damage_model == nil then
        return
    end
    if hp_data[1] ~= nil then
        ComponentSetValue2(damage_model, "hp", hp_data[1])
    end
    if hp_data[2] ~= nil then
        ComponentSetValue2(damage_model, "max_hp", hp_data[2])
    end
end

function util.lerp(a, b, alpha)
    return a * alpha + b * (1 - alpha)
end

return util