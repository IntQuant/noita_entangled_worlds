function temple_spawn_guardian( pos_x, pos_y )
    if GameHasFlagRun("ew_flag_this_is_host") then
        CrossCall("ew_spawn_stevari", pos_x, pos_y)
    end
end