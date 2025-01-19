local item = GetUpdatedEntityID()
if EntityGetRootEntity(item) == item then
    EntityKill(item)
end
