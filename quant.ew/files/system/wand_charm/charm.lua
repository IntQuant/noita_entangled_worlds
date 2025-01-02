dofile_once("data/scripts/animals/wand_charm.lua")
local ent = GetUpdatedEntityID()
local x, y = EntityGetTransform(ent)
material_area_checker_success(x, y)
