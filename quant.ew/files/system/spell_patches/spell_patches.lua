-- Stop death crosses from homing on other players.
util.replace_text_in("data/entities/projectiles/deck/death_cross.xml", 'detect_distance="350"', 'detect_distance="0"')
util.replace_text_in(
    "data/entities/projectiles/deck/death_cross_big.xml",
    'detect_distance="350"',
    'detect_distance="0"'
)

-- (Hopefully) Fix crash with spells_to_power and essence_to_power.
util.copy_file_content(
    "mods/quant.ew/files/system/spell_patches/spells_to_power.lua",
    "data/scripts/projectiles/spells_to_power.lua"
)
util.copy_file_content(
    "mods/quant.ew/files/system/spell_patches/essence_to_power.lua",
    "data/scripts/projectiles/essence_to_power.lua"
)

return {}
