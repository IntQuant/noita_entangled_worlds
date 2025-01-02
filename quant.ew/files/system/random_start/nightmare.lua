local orig = generate_gun

function generate_gun(cost, level, force_unshuffle)
    local sx, sy = CrossCall("ew_per_peer_seed")
    local entity_id = GetUpdatedEntityID()
    local x, y = EntityGetTransform(entity_id)
    SetRandomSeed(x + sx, y + sy)
    local orig_SetRandomSeed = SetRandomSeed
    function SetRandomSeed(a, b) end

    local gun = orig(cost, level, force_unshuffle)

    SetRandomSeed = orig_SetRandomSeed

    return gun
end
