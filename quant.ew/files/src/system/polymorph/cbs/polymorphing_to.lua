function polymorphing_to(string_entity_we_are_about_to_polymorph_to)
    print("poly_to "..string_entity_we_are_about_to_polymorph_to)
    CrossCall("ew_player_polymorphing_to", tonumber(string_entity_we_are_about_to_polymorph_to))
end
