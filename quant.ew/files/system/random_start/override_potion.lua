local orig_potion = potion_a_materials

function potion_a_materials()
    SetRandomSeed(4, -2 + CrossCall("ew_per_peer_seed"))
    local orig_SetRandomSeed = SetRandomSeed
    function SetRandomSeed(x, y) end

    local potion = orig_potion()

    SetRandomSeed = orig_SetRandomSeed

    return potion
end