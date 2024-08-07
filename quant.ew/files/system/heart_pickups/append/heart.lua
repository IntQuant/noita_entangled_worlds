local old_fn = item_pickup

function item_pickup(entity_item, entity_who_picked, name)
    GlobalsSetValue("ew_heart_pickup", "normal")

    old_fn(entity_item, entity_who_picked, name)
end
