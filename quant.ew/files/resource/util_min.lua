local mod = {}
function mod.do_i_own(ent)
    local gid
    for _, v in ipairs(EntityGetComponent(ent, "VariableStorageComponent") or {}) do
        if ComponentGetValue2(v, "name") == "ew_gid_lid" then
            gid = v
            break
        end
    end
    return gid == nil or ComponentGetValue2(gid, "value_bool")
end
return mod
