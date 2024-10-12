function temple_spawn_guardian( pos_x, pos_y )
    if CrossCall("ew_is_host") then
        CrossCall("ew_spawn_stevari", pos_x, pos_y)
    end
end