function throw_item(xi, yi, xf, yf)
    CrossCall("ew_heart_statue_throw", GetUpdatedEntityID())
end

function item_pickup(entity_item, entity_pickupper, item_name)
    CrossCall("ew_heart_statue_pickup", entity_item)
end
