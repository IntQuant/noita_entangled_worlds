function throw_item()
    GlobalsSetValue("ew_thrown", tostring(GetUpdatedEntityID()))
end

function item_pickup()
    GlobalsSetValue("ew_picked", tostring(GetUpdatedEntityID()))
end

function kick()
end