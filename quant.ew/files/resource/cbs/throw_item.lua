function throw_item()
    local ent = GetUpdatedEntityID()
    EntityRemoveComponent(ent, GetUpdatedComponentID())
    EntityKill(ent)
end
