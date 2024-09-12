
local function notload(content)
    ModTextFileSetContent("mods/quant.ew/tmp_load.lua", content)
    loadfile("mods/quant.ew/tmp_load.lua")()
    print("Dofiled stuff")
end

-- The root file that imported everything else.
-- TODO: maybe dofile should be patched?
EwImportRoot = "???"

print("Append for director helpers is running")

orig_do_mod_appends = do_mod_appends
do_mod_appends = function(filename, ...)
    -- do_mod_appends = orig_do_mod_appends
    orig_do_mod_appends(filename, ...)
    EwImportRoot = filename
    print("do_mod_appends "..filename)
end

function EwSpawnDispath(fn_name, ...)
    print("Called "..fn_name.." from "..EwImportRoot)
    return _G[fn_name](...)
end

orig_RegisterSpawnFunction = RegisterSpawnFunction
function RegisterSpawnFunction(color, fn_name)
    -- if fn_name == "init" then
    --     orig_RegisterSpawnFunction(color, fn_name)
    --     return
    -- end
    detour_fn_name = "ew_detour_"..fn_name
    print("Register", color, fn_name, detour_fn_name)
    notload([[
        function ]]..detour_fn_name..[[(...)
            print("called detour ]] ..detour_fn_name.. [[")
            return EwSpawnDispath("]] .. fn_name .. [[", ...)
        end
        orig_RegisterSpawnFunction(]]..color..[[, "]]..detour_fn_name..[[")
    ]])
end
