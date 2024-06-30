local adjust_damage = {}

function adjust_damage.adjust_damage(dmg, details)
    -- fire or toxic or poison damage
    if details == 16 or details == 32768 or details == 65536 then
        local player_count = tonumber(GlobalsGetValue("ew_player_count", "1"))
        return dmg / player_count
    else
        local has_glass_cannon = GlobalsGetValue("PERK_PICKED_GLASS_CANNON_PICKUP_COUNT", "0") ~= "0"
        if has_glass_cannon then
            return dmg * 5
        end
    end
    
    return dmg
end

return adjust_damage