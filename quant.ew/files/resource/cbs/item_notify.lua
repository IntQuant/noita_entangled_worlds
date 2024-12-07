function throw_item()
    CrossCall("ew_thrown", GetUpdatedEntityID())
end

function item_pickup()
    local ent = GetUpdatedEntityID()
    local com = EntityGetFirstComponentIncludingDisabled(ent, "ItemComponent")
    if com ~= nil then
        ComponentSetValue2(com, "permanently_attached", false)
    end
    CrossCall("ew_picked", ent)
end