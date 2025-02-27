local old = OnNotEnoughManaForAction
function OnNotEnoughManaForAction()
    local ent = GetUpdatedEntityID()
    if not EntityHasTag(ent, "ew_client") then
        old()
    end
end
