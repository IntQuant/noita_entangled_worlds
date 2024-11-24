local old = GameCreateSpriteForXFrames
function GameCreateSpriteForXFrames(filename, ...)
    if filename == "data/particles/spatial_map_player.png" then
        CrossCall("ew_place_player_on_map")
    else
        old(filename, ...)
    end
end