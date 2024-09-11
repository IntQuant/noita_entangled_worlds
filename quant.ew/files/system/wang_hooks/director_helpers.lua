local orig_RegisterSpawnFunction = RegisterSpawnFunction


-- The root file that imported everything else.
-- TODO: maybe dofile should be patched?
EwImportRoot = "???"

print("Append for director helpers is running")

orig_do_mod_appends = do_mod_appends
do_mod_appends = function(filename, ...)
    --do_mod_appends = orig_do_mod_appends
    orig_do_mod_appends(filename, ...)
    EwImportRoot = filename
    print("do_mod_appends "..filename)
end

local orig_do_mod_appends = do_mod_appends
function RegisterSpawnFunction(color, fn_name)
    local root_id = "???" -- TODO, this should be id (path) of a file that actually calls RegisterSpawnFunction, that we can dofile to call related functions manually.
    detour_fn_name = "ew_detour_"..fn_name
    _G[detour_fn_name] = function(...)
        print("Called "..fn_name.." from "..EwImportRoot)
        _G[fn_name](...)
    end
    orig_RegisterSpawnFunction(color, detour_fn_name)
end
