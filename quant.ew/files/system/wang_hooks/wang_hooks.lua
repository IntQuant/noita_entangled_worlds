local uniq_flags = dofile_once("mods/quant.ew/files/system/uniq_flags/uniq_flags.lua")

--local rpc = net.new_rpc_namespace()

local module = {}

util.copy_file_content("mods/quant.ew/files/system/wang_hooks/wang_scripts.csv", "data/scripts/wang_scripts.csv")

module.files_with_spawnhooks = {}

for line in
    string.gmatch(ModTextFileGetContent("mods/quant.ew/files/system/wang_hooks/files_with_spawnhooks.txt"), "(.-)\n")
do
    -- print("Interned", line)
    table.insert(module.files_with_spawnhooks, line)
end

local function detour_name(orig_fn_name)
    return "ew_detour_" .. orig_fn_name
end

local function generate_detour_fn(orig_fn_name)
    local detour_fn_name = detour_name(orig_fn_name)

    local detour_fn = "\n"
        .. "function "
        .. detour_fn_name
        .. [[(x, y, w, h, is_open_path)
    local entity_load_orig = EntityLoad
    local entity_load_camera_bound = EntityLoadCameraBound
    if CrossCall("ew_wang_detour", EW_CURRENT_FILE, "]]
        .. orig_fn_name
        .. [[", x, y, w, h, is_open_path) then
        EntityLoad = function(...) end
        EntityLoadCameraBound = function(...) end
    end

    --]]
        .. orig_fn_name
        .. [[(x, y, w, h, is_open_path)

    EntityLoad = entity_load_orig
    EntityLoadCameraBound = entity_load_camera_bound
end
    ]]

    return detour_fn
end

local allow_patching = {
    spawn_all_shopitems = { "data/scripts/biomes/mountain/mountain_hall.lua" },
    spawn_shopitem = {},
    spawn_statue = {},
    spawn_crate = {},
    -- That one in lava lake.
    spawn_corpse = {},
    spawn_boss_limbs_trigger = {},
    spawn_spirit_spawner = {},
    spawn_dragonspot = {},
    spawn_gate = {},
    spawn_essence = {},
    spawn_friend = {},
    spawn_killer = {},
    spawn_hanging_prop = {},
    spawn_worm_deflector = {},
    spawn_potion_mimic_empty = {},
    spawn_potion_mimic = {},
    spawn_book_barren = {},
    spawn_potion_beer = {},
    spawn_potion_milk = {},
    spawn_scorpion = {},
    spawn_puzzle_potion_mimics = {},
    spawn_boss = {},
    spawn_book = {},
    spawn_secret = {},
    spawn_fruit = {},
    spawn_specialshop = {},
    spawn_rock1 = {},
    spawn_rock2 = {},
    spawn_prize = {},
    spawn_physics_fungus = {},
    spawn_physics_acid_fungus = {},
    --spawn_chest = {},
    spawn_bbqbox = {},
    spawn_trapwand = {},
    spawn_turret = {},
    spawn_fungi = {},
    spawn_robots = {},
    spawn_nest = {},
    spawn_mouth = {},
    spawn_fungitrap = {},
    spawn_bunker = {},
    spawn_bunker2 = {},
    spawn_statue_hand = {},
    spawn_burning_barrel = {},
    spawn_electricity_trap = {},
    spawn_fish = {},
    --spawn_stones = {},
    --spawn_skulls = {},
    spawn_scavenger_party = {},
    load_furniture = {},
    load_furniture_bunk = {},
    spawn_bigfish = {},
    spawn_cook = {},
    spawn_ghost_crystal = {},
    spawn_rainbow_card = {},
    spawn_reward_wands = {},
    spawn_scavengers = {},
    spawn_small_animals = {},
    spawn_swing_puzzle_box = {},
    spawn_vasta_or_vihta = {},
    spawn_wheel = {},
    spawn_wheel_small = {},
    spawn_wheel_tiny = {},
    spawn_large_enemies = {},
}

local current_file

local function patch_fn(color, orig_fn_name)
    -- print(color, orig_fn_name)
    -- Seems like init is special and doesn't work with this detour approach.
    if orig_fn_name == "init" then
        return nil
    end

    if allow_patching[orig_fn_name] == nil or table.contains(allow_patching[orig_fn_name], current_file) then
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
    content = content .. "\n" .. 'EW_CURRENT_FILE="' .. filename .. '"\n'

    local wang_scripts = ModTextFileGetContent("data/scripts/wang_scripts.csv")

    for val in string.gmatch(wang_scripts, "ew_detour_(.-),") do
        -- print("Generating detour fn for", val)
        content = content .. generate_detour_fn(val)
    end

    -- content = content .. generate_detour_fn("spawn_small_enemies")
    -- content = content .. generate_detour_fn("spawn_big_enemies")
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
    local is_open_str = "false"
    if is_open_path then
        is_open_str = "true"
    end

    util.run_in_new_context(
        "function RegisterSpawnFunction(...) end\n"
            .. "dofile_once('"
            .. file
            .. "')\n"
            -- .. "print("..x..","..y..","..w..","..h..",'"..is_open_str.."')\n"
            .. fn
            .. "("
            .. x
            .. ","
            .. y
            .. ","
            .. w
            .. ","
            .. h
            .. ","
            .. is_open_str
            .. ")\n"
    )
end

local function run_spawn_fn_if_uniq(file, fn, x, y, w, h, is_open_path)
    -- Check if we have been called already.
    -- TODO: it's probably a bad idea to use run flags for that.
    -- file shouldn't be significant, as (fn, x, y) seem to be always unique
    async(function()
        local flag = "wspwn_" .. fn .. "_" .. x .. "_" .. y
        if uniq_flags.request_flag(flag) then
            run_spawn_fn(file, fn, x, y, w, h, is_open_path)
        end
    end)
end

util.add_cross_call("ew_wang_detour", function(file, fn, x, y, w, h, is_open_path)
    run_spawn_fn_if_uniq(file, fn, x, y, w, h, is_open_path)

    -- Make so that whatever items get spawned won't be free because they count as "stolen".
    if fn == "spawn_all_shopitems" then
        EntityLoad("data/entities/buildings/shop_hitbox.xml", x, y)
    end

    return false
end)

return module
