local orig = material_area_checker_success

local origl = EntityLoad
function EntityLoad(path, x, y)
    if path ~= "data/entities/items/pickup/goldnugget_200.xml" or CrossCall("ew_do_i_own", GetUpdatedEntityID()) then
        return origl(path, x, y)
    end
end

function material_area_checker_success(pos_x, pos_y)
    if CrossCall("ew_do_i_own", GetUpdatedEntityID()) then
        CrossCall("ew_charm_sync", GetUpdatedEntityID())
        return orig(pos_x, pos_y)
    end
end
