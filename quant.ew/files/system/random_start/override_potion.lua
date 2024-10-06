local orig_potion = potion_a_materials

function potion_a_materials()
    local sx, sy = CrossCall("ew_per_peer_seed")
    SetRandomSeed(4 + sx, -2 + sy)
    local orig_SetRandomSeed = SetRandomSeed
    function SetRandomSeed(x, y) end

    local potion = orig_potion()

    SetRandomSeed = orig_SetRandomSeed

    return potion
end