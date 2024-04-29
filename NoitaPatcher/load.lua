
-- You're supposed to `dofile_once("path/to/load.lua")` this file.


local orig_do_mod_appends = do_mod_appends

do_mod_appends = function(filename, ...)
    do_mod_appends = orig_do_mod_appends
    do_mod_appends(filename, ...)

    local noitapatcher_path = string.match(filename, "(.*)/load.lua")
    if not noitapatcher_path then
        print("Couldn't detect NoitaPatcher path")
    end

    __nsew_path = noitapatcher_path .. "/noitapatcher/nsew/"

    package.cpath = package.cpath .. ";./" .. noitapatcher_path .. "/?.dll"
    package.path = package.path .. ";./" .. noitapatcher_path .. "/?.lua"

    -- Lua's loader should now be setup properly:
    -- local np = require("noitapatcher")
    -- local nsew = require("noitapatcher.nsew")
end
