ModLuaFileAppend("data/scripts/director_helpers.lua", "mods/quant.ew/files/system/wang_hooks/director_helpers.lua")

local module = {}

local module.files_with_spawnhooks = {}

for line in string.gmatch(ModTextFileGetContent("mods/quant.ew/files/system/wang_hooks/files_with_spawnhooks.txt"), "(.-)\n") do
    -- print("Interned", line)
    table.insert(module.files_with_spawnhooks, line)
end

local function patch_file(filename)

end

function module.on_late_init()
    for _, filename in ipairs(module.files_with_spawnhooks) do
        patch_file(filename)
    end
end

return module