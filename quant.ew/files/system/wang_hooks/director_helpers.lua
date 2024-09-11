local orig_RegisterSpawnFunction = RegisterSpawnFunction

local orig_do_mod_appends = do_mod_appends

-- The root file that imported everything else.
-- TODO: maybe dofile should be patched?
EwImportRoot = "???"

orig_do_mod_appends = do_mod_appends
do_mod_appends = function(filename, ...)
    --do_mod_appends = orig_do_mod_appends
    orig_do_mod_appends(filename, ...)
    EwImportRoot = filename
end

function RegisterSpawnFunction(color, fn_name)
    local root_id = "???" -- TODO, this should be id (path) of a file that actually calls RegisterSpawnFunction, that we can dofile to call related functions manually.
    detour_fn_name = "ew_detour_"..fn_name
    _G[detour_fn_name] = function(...)
        _G[fn_name](...)
    end
    RegisterSpawnFunction(color, detour_fn_name)
end
