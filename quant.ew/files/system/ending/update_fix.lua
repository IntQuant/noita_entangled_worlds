local old = EntityGetWithTag
function EntityGetWithTag(tag)
    if tag == "player_unit" then
        local x, y = EntityGetTransform(GetUpdatedEntityID())
        return { EntityGetClosestWithTag(x, y, "ew_peer") }
    else
        return old(tag)
    end
end
