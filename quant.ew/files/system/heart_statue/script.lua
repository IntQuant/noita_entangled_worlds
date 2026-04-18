function throw_item(xi, yi, xf, yf)
    print_error("ew_heart_statue_throw")
    CrossCall("ew_heart_statue_throw", GetUpdatedEntityID())
end

function item_pickup()
    print_error("ew_heart_statue_pickup")
    CrossCall("ew_heart_statue_pickup")
end
