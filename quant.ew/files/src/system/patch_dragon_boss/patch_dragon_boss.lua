local util = dofile_once("mods/quant.ew/files/src/util.lua")

ModLuaFileAppend("data/scripts/buildings/dragonspot.lua", "mods/quant.ew/files/src/system/patch_dragon_boss/dragonspot_script.lua")
util.replace_text_in("data/entities/buildings/dragonspot.xml", "player_unit", "ew_peer")

local module = {}

return module