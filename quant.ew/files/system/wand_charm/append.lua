local orig = material_area_checker_success

function material_area_checker_success(pos_x, pos_y)
    CrossCall("ew_charm_sync", GetUpdatedEntityID())
    return orig(pos_x, pos_y)
end
