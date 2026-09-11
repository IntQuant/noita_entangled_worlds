-- Replaces the base pickup entirely. Its sound and music play on every peer, and
-- the fight starts on whichever peer owns Kolmi - both handled in kolmi.lua.
function item_pickup(entity_item)
    local x, y = EntityGetTransform(entity_item)
    CrossCall("ew_sampo_picked", x, y)
end
