local old = item_pickup
function item_pickup(ent, run)
    if run == nil then
        local gid
        for _, v in ipairs(EntityGetComponent(ent, "VariableStorageComponent") or {}) do
            if ComponentGetValue2(v, "name") == "ew_gid_lid" then
                gid = v
                break
            end
        end
        if gid ~= nil and not ComponentGetValue2(gid, "value_bool") then
            CrossCall("ew_spawn_kolmi", ComponentGetValue2(gid, "value_string"))
        end
    end
    old(ent)
end