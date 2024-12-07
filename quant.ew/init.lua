dofile_once("mods/quant.ew/NoitaPatcher/load.lua")
np = require("noitapatcher")

package.cpath = package.cpath .. ";./mods/quant.ew/?.dll"
package.path = package.path .. ";./mods/quant.ew/?.lua"
print(package.cpath)

dofile_once( "data/scripts/lib/utilities.lua" )

np.InstallShootProjectileFiredCallbacks()
np.EnableGameSimulatePausing(false)
np.InstallDamageDetailsPatch()
np.SilenceLogs("Warning - streaming didn\'t find any chunks it could stream away...\n")

ewext = require("ewext0")

-- Make some stuff global, as it's way too annoying to import each time.
constants = dofile_once("mods/quant.ew/files/core/constants.lua")
ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
util = dofile_once("mods/quant.ew/files/core/util.lua")
net = dofile_once("mods/quant.ew/files/core/net.lua")
inventory_helper = dofile_once("mods/quant.ew/files/core/inventory_helper.lua")
player_fns = dofile_once("mods/quant.ew/files/core/player_fns.lua")

dofile_once("mods/quant.ew/files/system/player/player_cosmetics.lua")

local perk_fns = dofile_once("mods/quant.ew/files/core/perk_fns.lua")

local version = ModDoesFileExist("mods/quant.ew/files/version.lua") and dofile_once("mods/quant.ew/files/version.lua")
        or "unknown (dev build)"
print("Noita EW version: "..version)

dofile_once("data/scripts/lib/coroutines.lua")

ModLuaFileAppend("data/scripts/gun/gun.lua", "mods/quant.ew/files/resource/append/gun.lua")
ModLuaFileAppend("data/scripts/gun/gun_actions.lua", "mods/quant.ew/files/resource/append/action_fix.lua")

ModMagicNumbersFileAdd("mods/quant.ew/files/magic.xml")

util.add_cross_call("ew_per_peer_seed", function()
    return tonumber(string.sub(ctx.my_id, 8, 12), 16), tonumber(string.sub(ctx.my_id, 12), 16)
end)

util.add_cross_call("ew_spectator", function()
    if ctx.spectating_over_peer_id == nil then
        return ctx.my_player.entity or EntityGetWithTag("player_unit")[1]
    else
        return ctx.players[ctx.spectating_over_peer_id].entity
    end
end)

if GameHasFlagRun("ending_game_completed") then
    ModTextFileSetContent("data/entities/misc/loose_chunks.xml", "<Entity/>")
    ModTextFileSetContent("data/entities/misc/loose_chunks_huge.xml", "<Entity/>")
    ModTextFileSetContent("data/entities/projectiles/deck/crumbling_earth_effect.xml", "<Entity/>")
end

local function load_modules()
    ctx.load_system("ewext_init")

    ctx.dofile_and_add_hooks("mods/quant.ew/files/system/item_sync.lua")

    ctx.dofile_and_add_hooks("mods/quant.ew/files/system/player_sync.lua")
    ctx.dofile_and_add_hooks("mods/quant.ew/files/system/enemy_sync.lua")


    if ctx.proxy_opt.game_mode == "shared_health" then
        ctx.load_system("damage")
        ctx.load_system("heart_pickups")
        ctx.load_system("patch_meat_biome")
        ctx.load_system("kivi_patch")
    end
    if ctx.proxy_opt.game_mode == "local_health" then
        ctx.load_system("local_health")
        ctx.load_system("notplayer_ai")
        ctx.load_system("spectator_helps")
        ctx.load_system("end_fight")
    end


    ctx.dofile_and_add_hooks("mods/quant.ew/files/system/nickname.lua")

    if ctx.proxy_opt.debug then
        ctx.dofile_and_add_hooks("mods/quant.ew/files/system/debug.lua")
    end

    ctx.load_system("fungal_shift")
    ctx.dofile_and_add_hooks("mods/quant.ew/files/system/weather_sync.lua")
    ctx.load_system("polymorph")

    ctx.load_system("world_sync")

    ctx.load_system("spawn_hooks")
    ctx.dofile_and_add_hooks("mods/quant.ew/files/system/proxy_info.lua")
    ctx.load_system("perk_patches")

    ctx.load_system("player_tether")

    ctx.load_system("kolmi")
    ctx.load_system("ending")
    ctx.load_system("spell_patches")
    ctx.load_system("enemy_scaling")

    ctx.load_system("patch_dragon_boss")

    ctx.load_system("player_arrows")
    ctx.load_system("player_ping")
    ctx.load_system("extra_genomes")
    ctx.load_system("game_effect_sync")
    ctx.load_system("orb_sync")
    ctx.load_system("flag_sync")
    ctx.load_system("essence_sync")
    ctx.load_system("spectate")
    ctx.load_system("effect_data_sync")
    if ctx.proxy_opt.item_dedup then
        ctx.load_system("gen_sync")
    end
    ctx.load_system("karl")
    ctx.load_system("remove_wand_sound")
    if ctx.proxy_opt.randomize_perks then
        ctx.load_system("randomize_perks")
    end
    ctx.load_system("streaming_sync")
    ctx.load_system("random_start")
    ctx.load_system("worms")
    ctx.load_system("wand_charm")
    ctx.load_system("stevari")
    ctx.load_system("angry_ghost_memory")
    ctx.load_system("gate_boss")
    ctx.load_system("tapion")
    ctx.load_system("world_sync_cuts")
    ctx.load_system("hamis")
    ctx.load_system("spell_refresh")
    ctx.load_system("shiny_orb")
    ctx.load_system("potion_mimic")
    ctx.load_system("map")
    ctx.load_system("homunculus")
    ctx.load_system("text")
    ctx.load_system("ragdoll_fix")
end

local function load_extra_modules()
    print("Starting to load extra stuff")
    for _, file in ipairs(ModLuaFileGetAppends("mods/quant.ew/files/api/extra_modules.lua")) do
        ctx.dofile_and_add_hooks(file)
    end
end

local function fire()
    local inventory_component = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "Inventory2Component")
    if inventory_component ~= nil then
        local last_switch = ComponentGetValue2(inventory_component, "mLastItemSwitchFrame")
        local switched_now = last_switch == GameGetFrameNum()
        local special_seed = tonumber(GlobalsGetValue("ew_player_rng", "0"))
        local fire_data = player_fns.make_fire_data(special_seed, ctx.my_player)
        if fire_data ~= nil then
            if switched_now then
                fire_data.switched_now = true
            end
            net.send_fire(fire_data)
        end
    end
end

function OnProjectileFired(shooter_id, projectile_id, initial_rng, position_x, position_y, target_x, target_y, send_message,
                           unknown1, multicast_index, unknown3)
    ctx.hook.on_projectile_fired(shooter_id, projectile_id, initial_rng, position_x, position_y, target_x, target_y, send_message, unknown1, multicast_index, unknown3)
    if not EntityHasTag(shooter_id, "player_unit") and not EntityHasTag(shooter_id, "ew_client") then
        return -- Not fired by player, we don't care about it (for now?)
    end
    EntityAddTag(projectile_id, "ew_no_enemy_sync")
    local projectileComponent = EntityGetFirstComponentIncludingDisabled(projectile_id, "ProjectileComponent")
    local entity_that_shot
    if projectileComponent == nil then
        entity_that_shot = GameGetWorldStateEntity()
    else
        entity_that_shot = ComponentGetValue2(projectileComponent, "mEntityThatShot")
    end

    local shooter_player_data = player_fns.get_player_data_by_local_entity_id(shooter_id)
    local rng = 0
    -- Was shot locally
    if shooter_id == ctx.my_player.entity then
        -- If it was an initial shot by host
        if (entity_that_shot == 0 and multicast_index ~= -1 and unknown3 == 0) then
            if tonumber(GlobalsGetValue("ew_wand_fired", "0")) ~= 0 then
                rng = initial_rng
                table.insert(shooter_player_data.projectile_rng_init, rng)
            else
                rng = (shooter_player_data.projectile_seed_chain[shooter_id - 1] or 0) + 25
            end
        else
            rng = (shooter_player_data.projectile_seed_chain[entity_that_shot] or 0) + 25
        end
    else
        if (entity_that_shot == 0 and multicast_index ~= -1 and unknown3 == 0) then
            if #shooter_player_data.projectile_rng_init > 0 then
                rng = table.remove(shooter_player_data.projectile_rng_init, 1)
            else
                rng = (shooter_player_data.projectile_seed_chain[shooter_id - 1] or 0) + 25
            end
        else
            rng = (shooter_player_data.projectile_seed_chain[entity_that_shot] or 0) + 25
        end
    end
    shooter_player_data.projectile_seed_chain[shooter_id - 1] = rng
    shooter_player_data.projectile_seed_chain[entity_that_shot] = rng
    shooter_player_data.projectile_seed_chain[projectile_id] = rng
    for _, lua in ipairs(EntityGetComponent(projectile_id, "LuaComponent") or {}) do
        if ComponentGetValue2(lua, "script_source_file") == "data/scripts/projectiles/transmutation.lua" then
            EntityAddComponent2(projectile_id, "VariableStorageComponent", {name = "ew_transmutation", value_int = rng})
            break
        end
    end
    local n = EntityGetFilename(projectile_id)
    if n == "data/entities/items/pickup/egg_hollow.xml" then
        EntityAddComponent2(projectile_id, "VariableStorageComponent", {_tags = "ew_egg", value_int = rng})
        EntityAddComponent2(projectile_id, "VariableStorageComponent", {_tags="ew_global_item_id",
                                                                        value_string = shooter_player_data.peer_id .. ":" .. rng})
        EntityAddTag(projectile_id, "ew_global_item")
    elseif n == "data/entities/projectiles/deck/rock.xml" then
        EntityAddComponent2(projectile_id, "VariableStorageComponent", {_tags="ew_global_item_id",
                                                                        value_string = shooter_player_data.peer_id .. ":" .. rng})
        EntityAddTag(projectile_id, "ew_global_item")
        EntityAddTag(projectile_id, "ew_no_spawn")
    end
    np.SetProjectileSpreadRNG(rng)
end

--function OnProjectileFiredPost(shooter_id, projectile_id, rng, position_x, position_y, target_x, target_y, send_message,
--    unknown1, multicast_index, unknown3)
--end

function OnPausedChanged(paused, is_wand_pickup)
    ctx.is_paused = paused
    ctx.is_wand_pickup = is_wand_pickup
    local players = EntityGetWithTag("player_unit") or {}

    if (players[1]) then
        np.RegisterPlayerEntityId(players[1])
        --local inventory_gui = EntityGetFirstComponentIncludingDisabled(players[1], "InventoryGuiComponent")
        local controls_component = EntityGetFirstComponentIncludingDisabled(players[1], "ControlsComponent")
        if (paused) then
            --EntitySetComponentIsEnabled(players[1], inventory_gui, false)
            np.EnableInventoryGuiUpdate(false)
            np.EnablePlayerItemPickUpper(false)
            ComponentSetValue2(controls_component, "enabled", true)
        else
            --EntitySetComponentIsEnabled(players[1], inventory_gui, true)
            np.EnableInventoryGuiUpdate(true)
            np.EnablePlayerItemPickUpper(true)
            ComponentSetValue2(controls_component, "enabled", true)
        end
    end
end

function OnWorldInitialized() -- This is called once the game world is initialized. Doesn't ensure any world chunks actually exist. Use OnPlayerSpawned to ensure the chunks around player have been loaded or created.
    if ctx.is_host then
        GameAddFlagRun("ew_flag_this_is_host")
    else
        GameRemoveFlagRun("ew_flag_this_is_host")
    end
    ctx.hook.on_world_initialized()
end

local last_chunk

local last_flex

function OnPlayerSpawned( player_entity ) -- This runs when player entity has been created
    print("Initial player entity: "..player_entity)

    if GlobalsGetValue("ew_player_count", "") == "" then
        GlobalsSetValue("ew_player_count", "1")
    end

    local x, y = EntityGetTransform(player_entity)
    ctx.initial_player_pos = {x=x, y=y}

    local my_player = player_fns.make_playerdata_for(player_entity, ctx.my_id)
    ctx.players[ctx.my_id] = my_player
    ctx.player_data_by_local_entity[player_entity] = my_player
    ctx.ready = true
    ctx.my_player = my_player

    EntityAddTag(player_entity, "ew_peer")

    if not GameHasFlagRun("ew_flag_notplayer_active") then
        EntityAddComponent2(player_entity, "LuaComponent", {script_wand_fired = "mods/quant.ew/files/resource/cbs/count_times_wand_fired.lua"})
    end

    net.send_welcome()

    local item_pick = EntityGetFirstComponentIncludingDisabled(player_entity, "ItemPickUpperComponent")
    if item_pick ~= nil then
        ComponentSetValue2(item_pick, "is_immune_to_kicks", true)
    end

    ctx.hook.on_local_player_spawn(my_player)
    ctx.hook.on_should_send_updates()

    GamePrint("Noita Entangled Worlds version "..version)

    OnPausedChanged(false, false)

    print("Game state entity: "..GameGetWorldStateEntity())

    if not GameHasFlagRun("ew_flag_notplayer_active") then
        player_cosmetics(player_entity)
        player_color(player_entity)
    else
        EntityInflictDamage(player_entity, 1000000, "DAMAGE_CURSE", "dont rejoin", "NONE", 0, 0, GameGetWorldStateEntity())
        GameAddFlagRun("ew_kill_player")
    end
    if ctx.host_id == ctx.my_id then
        last_chunk = tonumber(ModSettingGet("quant.ew.chunk_target")) or 24
        np.MagicNumbersSetValue("STREAMING_CHUNK_TARGET", last_chunk)
    end
    last_flex = ModSettingGet("quant.ew.flex")
    np.MagicNumbersSetValue("GRID_FLEXIBLE_MAX_UPDATES", last_flex)
    local controls_component = EntityGetFirstComponentIncludingDisabled(player_entity, "ControlsComponent")
    ComponentSetValue2(controls_component, "enabled", true)
    for _, child in ipairs(EntityGetAllChildren(player_entity) or {}) do
        local com = EntityGetFirstComponentIncludingDisabled(child, "LuaComponent")
        if com ~= nil and ComponentGetValue2(com, "script_source_file") == "data/scripts/perks/map.lua" then
            EntityRemoveComponent(child, com)
            EntityAddComponent2(child, "LuaComponent", {script_source_file = "data/scripts/perks/map.lua"})
            return
        end
    end
end

local function change_homing(x, y)
    for _, proj in pairs(EntityGetInRadiusWithTag(x, y, 512, "player_projectile")) do
        local homing = EntityGetFirstComponentIncludingDisabled(proj, "HomingComponent")
        if homing ~= nil then
            local projcom = EntityGetFirstComponentIncludingDisabled(proj, "ProjectileComponent")
            if projcom ~= nil then
                local whoshot = ComponentGetValue2(projcom, "mWhoShot")
                if EntityHasTag(whoshot, "ew_notplayer") or GameHasFlagRun("ending_game_completed") then
                    ComponentSetValue2(homing, "target_tag", "ew_peer")
                end
            end
        end
    end
end

local function on_world_pre_update_inner()
    if ctx.my_player == nil or ctx.my_player.entity == nil then return end

    GlobalsSetValue("ew_player_rng", tostring(GameGetFrameNum()))

    if not ctx.run_ended then
        net.update()
    end

    local inventory_gui_comp = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "InventoryGuiComponent")
    if inventory_gui_comp and inventory_gui_comp ~= 0 then
        local inventory_open = ComponentGetValue2(inventory_gui_comp, "mActive")
        if ctx.is_inventory_open and not inventory_open then
            ctx.events.inventory_maybe_just_changed = true
        end
        ctx.is_inventory_open = inventory_open
    end

    if GameGetFrameNum() % 120 == 76 and not ctx.run_ended then
        if ctx.host_id == ctx.my_id then
            local new_chunk = tonumber(ModSettingGet("quant.ew.chunk_target")) or 24
            if last_chunk ~= new_chunk then
                last_chunk = new_chunk
                np.MagicNumbersSetValue("STREAMING_CHUNK_TARGET", last_chunk)
            end
        end
        local new_flex = ModSettingGet("quant.ew.flex")
        if new_flex ~= last_flex then
            last_flex = new_flex
            np.MagicNumbersSetValue("GRID_FLEXIBLE_MAX_UPDATES", last_flex)
        end
        player_fns.respawn_if_necessary()
    end

    local sha_check = GameGetFrameNum() % 5 == 0 and inventory_helper.has_inventory_changed(ctx.my_player)
    if ctx.events.new_player_just_connected or ctx.events.inventory_maybe_just_changed or sha_check then
        local inventory_state, spells = player_fns.serialize_items(ctx.my_player)
        if inventory_state ~= nil then
            net.send_player_inventory(inventory_state, spells)
        end
    end

    -- Perk sync
    if GameGetFrameNum() % 120 == 10 and not ctx.run_ended then
        local perk_data = perk_fns.get_my_perks()
        if perk_data ~= nil then
            net.send_player_perks(perk_data)
        end
    end

    if not ctx.run_ended then
        if ctx.is_host then
            ctx.hook.on_world_update_host()
        else
            ctx.hook.on_world_update_client()
        end
        ctx.hook.on_world_update()
    end

    if GameGetFrameNum() % 4 == 0 then
        local x, y = EntityGetTransform(ctx.my_player.entity)
        if x ~= nil then
            change_homing(x, y)
        end
    elseif GameGetFrameNum() % 4 == 1 then
        local x, y = EntityGetTransform(ctx.my_player.entity)
        local cx, cy = GameGetCameraPos()
        if x ~= nil and cx ~= nil then
            if math.abs(x - cx) > 256 or math.abs(y - cy) > 256 then
                change_homing(cx, cy)
            end
        end
    end

    perk_fns.on_world_update()
    wake_up_waiting_threads(1)
end

function OnWorldPreUpdate() -- This is called every time the game is about to start updating the world
    if  net.connect_failed then
        if GameGetFrameNum() % 180 == 0 then
            GamePrint("Entangled Worlds mod is enabled, but it couldn't connect to proxy!")
            GamePrint("You need to start the proxy and join the lobby first.")
            GamePrint("If you want to play singleplayer, disable the mod and start a new game.")
        end
        return
    end
    util.tpcall(on_world_pre_update_inner)
end

local function on_world_post_update_inner()
    if ctx.my_player == nil or ctx.my_player.entity == nil then return end

    if not ctx.run_ended then
        ctx.hook.on_world_update_post()
    end

    local times_wand_fired = tonumber(GlobalsGetValue("ew_wand_fired", "0"))
    GlobalsSetValue("ew_wand_fired", "0")
    local wand = player_fns.get_active_held_item(ctx.my_player.entity)
    local ability
    if wand ~= nil and EntityHasTag(wand, "card_action") then
        ability = EntityGetFirstComponentIncludingDisabled(wand, "AbilityComponent")
    end
    if times_wand_fired > 0
            or (ability ~= nil and ComponentGetValue2(ability, "mCastDelayStartFrame") == GameGetFrameNum()) then
        fire()
    end
end

function OnWorldPostUpdate() -- This is called every time the game has finished updating the world
    if  net.connect_failed then
        return
    end
    util.tpcall(on_world_post_update_inner)
    ctx.events = {}
    net.proxy_send("flush", "")
end

function register_localizations(translation_file, clear_count)

    clear_count = clear_count or 0

    local loc_content = ModTextFileGetContent("data/translations/common.csv") -- Gets the original translations of the game

    local append_content = ModTextFileGetContent(translation_file) -- Gets my own translations file

    -- Split the append_content into lines
    local lines = {}
    for line in append_content:gmatch("[^\n]+") do
        table.insert(lines, line)
    end

    -- Remove the first clear_count lines
    for i = 1, clear_count do
        table.remove(lines, 1)
    end

    -- Reconstruct append_content after removing clear_count lines
    local new_append_content = table.concat(lines, "\n")

    -- if loc_content does not end with a new line, add one
    if not loc_content:match("\n$") then
        loc_content = loc_content .. "\n"
    end

    -- Concatenate loc_content and new_append_content without extra newline character
    local new_content = loc_content .. new_append_content .. "\n"

    -- Set the new content to the file
    ModTextFileSetContent("data/translations/common.csv", new_content)
end

function OnModPreInit()
    register_localizations("mods/quant.ew/translations.csv", 1)
    ctx.init()
    net.init()

    if not net.connect_failed then
        load_modules()
        print("Entangled worlds init ok")
    end
end

function OnModInit()
    load_extra_modules()
end

function OnModPostInit()
    ctx.hook.on_late_init()
end

function OnPlayerDied(player_entity)
    ctx.hook.on_player_died(player_entity)
    print("player died")
end