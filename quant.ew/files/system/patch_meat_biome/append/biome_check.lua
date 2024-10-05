function biome_entered( new_biome_name, old_biome_name )
    -- print( "new_biome_name: " .. new_biome_name )
    local e = GetUpdatedEntityID()

    if( new_biome_name == "$biome_meat" ) then
        EntityAddTag(e, "ew_no_heal_in_meat_biome")
    else
        EntityRemoveTag(e, "ew_no_heal_in_meat_biome")
    end
end