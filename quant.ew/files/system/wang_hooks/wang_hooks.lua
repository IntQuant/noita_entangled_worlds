local rpc = net.new_rpc_namespace()

local module = {}

util.copy_file_content("mods/quant.ew/files/system/wang_hooks/wang_scripts.csv", "data/scripts/wang_scripts.csv")

module.files_with_spawnhooks = {}

for line in string.gmatch(ModTextFileGetContent("mods/quant.ew/files/system/wang_hooks/files_with_spawnhooks.txt"), "(.-)\n") do
    -- print("Interned", line)
    table.insert(module.files_with_spawnhooks, line)
end

local function detour_name(orig_fn_name)
    return "ew_detour_" .. orig_fn_name
end

local function generate_detour_fn(orig_fn_name)
    local detour_fn_name = detour_name(orig_fn_name)

    local detour_fn = "\n".."function " .. detour_fn_name .. [[(x, y, w, h, is_open_path)
    local entity_load_orig = EntityLoad
    local entity_load_camera_bound = EntityLoadCameraBound
    if CrossCall("ew_wang_detour", EW_CURRENT_FILE, "]]..orig_fn_name..[[", x, y, w, h, is_open_path) then
        EntityLoad = function(...) end
        EntityLoadCameraBound = function(...) end
    end

    ]] .. orig_fn_name .. [[(x, y, w, h, is_open_path)

    EntityLoad = entity_load_orig
    EntityLoadCameraBound = entity_load_camera_bound
end
    ]]

    return detour_fn
end

local function patch_fn(color, orig_fn_name)
    -- print(color, orig_fn_name)
    -- Seems like init is special and doesn't work with this detour approach.
    if orig_fn_name == "init" then
        return nil
    end

    if orig_fn_name ~= "spawn_all_shopitems" then
        return nil
    end

    local detour_fn_name = detour_name(orig_fn_name)
    local detour_fn = generate_detour_fn(orig_fn_name)
    
    local new_fn_call = "RegisterSpawnFunction( " .. color .. ', "' .. detour_fn_name .. '" )'
    local repl = new_fn_call .. "\n" .. detour_fn
    -- print(repl)
    return repl
end

local function patch_file(filename)
    print("Patching", filename)
    local content = ModTextFileGetContent(filename)
    current_file = filename
    -- A textbook example of how to NOT use regular expressions.
    content = string.gsub(content, 'RegisterSpawnFunction[(][ ]?(.-), "(.-)"[ ]?[)]', patch_fn)
    content = content .. "\n"..'EW_CURRENT_FILE="'..filename..'"\n'

    content = content .. generate_detour_fn("spawn_small_enemies")
    content = content .. generate_detour_fn("spawn_big_enemies")
    -- content = content .. generate_detour_fn("spawn_items")

    ModTextFileSetContent(filename, content)
end

function module.on_late_init()
    for _, filename in ipairs(module.files_with_spawnhooks) do
        if string.sub(filename, 1, 1) ~= "#" then
            patch_file(filename)
        end
    end
end

-- Runs a wang spawn fn if it wasn't called for these coordinates yet.
local function run_spawn_fn(file, fn, x, y, w, h, is_open_path)
    -- Check if we have been called already.
    -- TODO: it's probably a bad idea to use run flags for that.
    -- file shouldn't be significant, as (fn, x, y) seem to be always unique
    local flag = "wspwn_"..fn.."_"..x.."_"..y
    if GameHasFlagRun(flag) then
        return
    end
    GameAddFlagRun(flag)

    local is_open_str = "false"
    if is_open_path then
        is_open_str = "true"
    end
    
    util.run_in_new_context(
        "function RegisterSpawnFunction(...) end\n"
        .. "dofile_once('"..file.."')\n"
        -- .. "print("..x..","..y..","..w..","..h..",'"..is_open_str.."')\n"
        .. fn.."("..x..","..y..","..w..","..h..","..is_open_str..")\n"
    )
end

rpc.opts_reliable()
function rpc.run_spawn_fn(file, fn, x, y, w, h, is_open_path)
    if ctx.is_host then
        run_spawn_fn(file, fn, x, y, w, h, is_open_path)
    end
end


util.add_cross_call("ew_wang_detour", function(file, fn, x, y, w, h, is_open_path)
    if ctx.is_host then
        run_spawn_fn(file, fn, x, y, w, h, is_open_path)
    else
        rpc.run_spawn_fn(file, fn, x, y, w, h, is_open_path)
    end

    return true
end)

return module
