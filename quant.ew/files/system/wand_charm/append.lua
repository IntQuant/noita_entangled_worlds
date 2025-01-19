local orig = material_area_checker_success

function material_area_checker_success(pos_x, pos_y)
    if CrossCall("ew_do_i_own", GetUpdatedEntityID()) then
        CrossCall("ew_charm_sync", GetUpdatedEntityID())
        return orig(pos_x, pos_y)
    end
end
