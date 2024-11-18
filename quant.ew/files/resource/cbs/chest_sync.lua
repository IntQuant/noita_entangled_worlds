local old = on_open
function on_open(entity_item, dont)
    if dont == nil then
        CrossCall("ew_chest_opened", entity_item)
    end
    old(entity_item)
end

function init(entity_id)
    on_open(entity_id, true)
    EntityKill(entity_id)
end