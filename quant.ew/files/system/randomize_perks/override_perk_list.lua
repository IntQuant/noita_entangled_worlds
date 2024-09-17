local orig_perk_get_spawn_order = perk_get_spawn_order

function perk_get_spawn_order( ignore_these_ )
    SetRandomSeed(1, 2 + CrossCall("ew_per_peer_seed"))
    local orig_SetRandomSeed = SetRandomSeed
    function SetRandomSeed(x, y) end
    
    orig_perk_get_spawn_order( ignore_these_ )

    SetRandomSeed = orig_SetRandomSeed
end