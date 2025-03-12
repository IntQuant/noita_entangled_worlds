local mod = {}
function mod.do_i_own(ent)
    if ent == nil or not EntityGetIsAlive(ent) then
        return false
    end
    local gid
    for _, v in ipairs(EntityGetComponentIncludingDisabled(ent, "VariableStorageComponent") or {}) do
        if ComponentGetValue2(v, "name") == "ew_gid_lid" then
            gid = v
            break
        end
    end
    return gid == nil or ComponentGetValue2(gid, "value_bool")
end
return mod
