local hp_scale = ctx.proxy_opt.enemy_hp_scale

if hp_scale > 1.01 then
    SessionNumbersSetValue("DESIGN_SCALE_ENEMIES", "1")
    SessionNumbersSetValue("DESIGN_NEW_GAME_PLUS_HP_SCALE_MIN", hp_scale)
    SessionNumbersSetValue("DESIGN_NEW_GAME_PLUS_HP_SCALE_MAX", hp_scale)
end

return {}
