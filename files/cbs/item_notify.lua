function throw_item()
    GamePrint("Item thrown")
    GlobalsSetValue("ew_thrown", tostring(GetUpdatedEntityID()))
end

function item_pickup()
    GamePrint("Item pickup")
    GlobalsSetValue("ew_picked", tostring(GetUpdatedEntityID()))
end

function kick()
    -- GamePrint("Item kicked")
end