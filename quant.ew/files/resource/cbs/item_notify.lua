function throw_item()
    GlobalsSetValue("ew_thrown", tostring(GetUpdatedEntityID()))
end

function item_pickup()
    local ent = GetUpdatedEntityID()
    local com = EntityGetFirstComponentIncludingDisabled(ent, "ItemComponent")
    if com ~= nil then
        ComponentSetValue2(com, "permanently_attached", false)
    end
    GlobalsSetValue("ew_picked", tostring(ent))
end