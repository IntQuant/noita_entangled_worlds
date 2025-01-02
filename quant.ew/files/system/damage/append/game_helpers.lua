function heal_entity(entity_id, heal_amount)
    CrossCall("ew_ds_damaged", -heal_amount, "healed", entity_id, true)
    -- gfx effect
    local x, y = EntityGetTransform(entity_id)
    local entity_fx = EntityLoad("data/entities/particles/heal_effect.xml", x, y)
    EntityAddChild(entity_id, entity_fx)
end
