local old = item_pickup
function item_pickup(entity_item, entity_who_picked, name)
    old(entity_item, entity_who_picked, name)
    CrossCall("ew_refresh_inventory")
end
