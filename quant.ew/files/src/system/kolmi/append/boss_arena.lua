local old_spawn_items = spawn_items

function spawn_items(x, y)
    old_spawn_items(x, y)
    
    CrossCall("ew_sampo_spawned")
end