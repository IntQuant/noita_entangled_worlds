function throw_item(xi, yi, xf, yf)
    local dx = xf - xi
    local dy = yf - yi
    CrossCall("ew_potion_mimic_throw", GetUpdatedEntityID(), dx / 8, dy / 8)
end

function item_pickup()
    CrossCall("ew_potion_mimic_pickup")
end