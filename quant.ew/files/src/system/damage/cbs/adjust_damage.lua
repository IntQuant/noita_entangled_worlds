local adjust_damage = {}

function adjust_damage.adjust_damage(dmg, details)
    -- fire or toxic or poison damage
    if details == 16 or details == 32768 or details == 65536 then
        local player_count = tonumber(GlobalsGetValue("ew_player_count", "1"))
        return dmg / player_count
    end
    
    return dmg
end

return adjust_damage