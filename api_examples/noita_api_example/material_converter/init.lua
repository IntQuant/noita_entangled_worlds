package.cpath = package.cpath .. ";./mods/material_converter/?.dll"
package.path = package.path .. ";./mods/material_converter/?.lua"
local material_converter = require("material_converter")
function OnWorldInitialized()
    local water = CellFactory_GetType("water")
    local lava = CellFactory_GetType("lava")
    material_converter.convert(water,lava)
end
