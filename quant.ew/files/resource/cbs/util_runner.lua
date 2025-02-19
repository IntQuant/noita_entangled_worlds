-- Used for util.run_in_new_context(code)
local f = loadfile("data/ew_code_tmp.lua")
local ok = pcall(f)
if not ok then
    print("Failed to call code passed to util.run_in_new_context")
    print(ModTextFileGetContent("data/ew_code_tmp.lua"))
end
