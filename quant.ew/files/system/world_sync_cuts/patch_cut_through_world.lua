local old_GameCutThroughWorldVertical = GameCutThroughWorldVertical

function GameCutThroughWorldVertical(...)
    old_GameCutThroughWorldVertical(...)
    CrossCall("ew_cut_through_world", ...)
end
