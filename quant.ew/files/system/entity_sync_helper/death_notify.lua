function death(damage_type_bit_field, damage_message, entity_thats_responsible, drop_items)
    local ent = GetUpdatedEntityID()
    local x, y = EntityGetTransform(ent)
    local wait_on_kill = false
    local damage = EntityGetFirstComponentIncludingDisabled(ent, "DamageModelComponent")
    if damage ~= nil then
        wait_on_kill = ComponentGetValue2(damage, "wait_for_kill_flag_on_death")
    end
    local drops_gold = false
    for _, lua in ipairs(EntityGetComponent(ent, "LuaComponent") or {}) do
        if ComponentGetValue2(lua, "script_death") == "data/scripts/items/drop_money.lua" then
            drops_gold = true
            break
        end
    end
    if EntityGetFirstComponentIncludingDisabled(ent, "VariableStorageComponent", "no_gold_drop") ~= nil then
        drops_gold = false
    end
    CrossCall("ew_death_notify", ent, wait_on_kill, drops_gold, x, y, EntityGetFilename(ent), entity_thats_responsible)
end
