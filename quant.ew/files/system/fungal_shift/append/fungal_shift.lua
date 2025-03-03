local old_fungal_shift = fungal_shift

function fungal_shift(entity, x, y, debug_no_limits)
    local old_convert_mat = ConvertMaterialEverywhere
    local old_get_text = GameTextGet
    local from_material_name = ""
    local changed_anything = false
    function ConvertMaterialEverywhere(mat_from, mat_to)
        old_convert_mat(mat_from, mat_to)
        changed_anything = true
        CrossCall("ew_fungal_shift_conversion", mat_from, mat_to)
    end
    function GameTextGet(arg1, arg2, arg3, arg4)
        if arg1 == "$logdesc_reality_mutation" then
            from_material_name = arg2
        end
        if arg3 == nil then
            return old_get_text(arg1, arg2)
        elseif arg4 == nil then
            return old_get_text(arg1, arg2, arg3)
        else
            return old_get_text(arg1, arg2, arg3, arg4)
        end
    end
    old_fungal_shift(entity, x, y, debug_no_limits)
    ConvertMaterialEverywhere = old_convert_mat
    GameTextGet = old_get_text
    local iter = tonumber(GlobalsGetValue("fungal_shift_iteration", "0"))
    if changed_anything then
        CrossCall("ew_fungal_shift", iter, from_material_name)
    end
end
