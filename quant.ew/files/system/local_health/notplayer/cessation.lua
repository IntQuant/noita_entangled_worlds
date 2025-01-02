local ent = GetUpdatedEntityID()
for _, child in ipairs(EntityGetAllChildren(ent) or {}) do
    if EntityGetName(child) == "cursor" then
        EntitySetComponentIsEnabled(child, EntityGetFirstComponentIncludingDisabled(child, "SpriteComponent"), false)
    end
end
