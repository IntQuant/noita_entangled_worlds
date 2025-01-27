local old = item_pickup
function item_pickup(ent, run)
    if run == nil then
        CrossCall("ew_spawn_kolmi", ent)
    end
    old(ent)
end