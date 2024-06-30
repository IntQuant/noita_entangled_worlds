local util = dofile_once("mods/quant.ew/files/src/util.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local np = require("noitapatcher")

local module = {}

ModLuaFileAppend("data/scripts/perks/perk_list.lua", "mods/quant.ew/files/src/system/perk_patches/append/perk_list.lua")


return module
