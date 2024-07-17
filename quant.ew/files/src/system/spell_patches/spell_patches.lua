local util = dofile_once("mods/quant.ew/files/src/util.lua")

util.replace_text_in("data/entities/projectiles/deck/death_cross.xml", 'detect_distance="350"', 'detect_distance="0"')
util.replace_text_in("data/entities/projectiles/deck/death_cross_big.xml", 'detect_distance="350"', 'detect_distance="0"')
