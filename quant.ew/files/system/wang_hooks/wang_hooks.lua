--ModLuaFileAppend("data/scripts/director_helpers.lua", "mods/quant.ew/files/system/wang_hooks/director_helpers.lua")

local module = {}

module.files_with_spawnhooks = {}

for line in string.gmatch(ModTextFileGetContent("mods/quant.ew/files/system/wang_hooks/files_with_spawnhooks.txt"), "(.-)\n") do
    -- print("Interned", line)
    table.insert(module.files_with_spawnhooks, line)
end

local current_file = nil

local function patch_fn(color, orig_fn_name)
    -- print(color, orig_fn_name)
    local current_file = current_file

    detour_fn_name = "ew_detour_" .. orig_fn_name

    detour_fn = "function " .. detour_fn_name .. [[(...)
        print("Called", "]] .. current_file .. [[", "]] .. detour_fn_name .. [[")
        ]] .. orig_fn_name .. [[(...)
    end
    ]]
    
    new_fn_call = "RegisterSpawnFunction( " .. color .. ', "' .. detour_fn_name .. '" )'
    repl = new_fn_call .. "\n" .. detour_fn
    -- print(repl)
    return repl
end

local function patch_file(filename)
    print("Patching", filename)
    local content = ModTextFileGetContent(filename)
    current_file = filename
    -- A textbook example of how to NOT use regular expressions.
    content = string.gsub(content, 'RegisterSpawnFunction[(][ ]?(.-), "(.-)"[ ]?[)]', patch_fn)
    ModTextFileSetContent(filename, content)
end

function module.on_late_init()
    for _, filename in ipairs(module.files_with_spawnhooks) do
        patch_file(filename)
    end
end

return module
