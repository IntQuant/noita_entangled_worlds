local old = on_open
function on_open(entity_item, dont)
    if dont == nil then
        local x, y = EntityGetTransform(entity_item)
        local position_comp = EntityGetFirstComponent(entity_item, "PositionSeedComponent")
        local rand_x = -1
        local rand_y = -1
        if position_comp then
            rand_x = tonumber(ComponentGetValue(position_comp, "pos_x"))
            rand_y = tonumber(ComponentGetValue(position_comp, "pos_y"))
        end
        CrossCall("ew_chest_opened", x, y, rand_x, rand_y, EntityGetFilename(entity_item), entity_item)
    end
    if not CrossCall("ew_has_opened_chest", entity_item) then
        old(entity_item)
    end
end

function init(entity_id)
    if not CrossCall("ew_has_opened_chest", entity_item) then
        on_open(entity_id, true)
    end
    EntityKill(entity_id)
end
