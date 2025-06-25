local old = on_open
function on_open(entity_item)
    local gid
    for _, v in ipairs(EntityGetComponent(entity_item, "VariableStorageComponent") or {}) do
        if ComponentGetValue2(v, "name") == "ew_gid_lid" then
            gid = v
            break
        end
    end
    if gid ~= nil then
        local rand_x, rand_y = -1, -1
        local x, y = EntityGetTransform(entity_item)
        local position_comp = EntityGetFirstComponent(entity_item, "PositionSeedComponent")
        if position_comp ~= nil then
            rand_x = tonumber(ComponentGetValue(position_comp, "pos_x"))
            rand_y = tonumber(ComponentGetValue(position_comp, "pos_y"))
        end
        local is_mine = false
        if ComponentGetValue2(gid, "value_bool") then
            is_mine = true
        end
        gid = ComponentGetValue2(gid, "value_string")
        CrossCall("ew_chest_opened", x, y, rand_x, rand_y, EntityGetFilename(entity_item), gid, is_mine)
    else
        old(entity_item)
        EntityKill(entity_item)
    end
end

function init(entity_id)
    if
        ComponentGetValue2(GetUpdatedComponentID(), "call_init_function")
        and ComponentGetValue2(GetUpdatedComponentID(), "execute_on_added")
    then
        old(entity_id)
        EntityKill(entity_id)
    end
end
