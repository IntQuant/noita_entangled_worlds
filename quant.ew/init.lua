dofile_once("mods/quant.ew/NoitaPatcher/load.lua")
local np = require("noitapatcher")

dofile_once( "data/scripts/lib/utilities.lua" )

np.InstallShootProjectileFiredCallbacks()
np.EnableGameSimulatePausing(false)
np.InstallDamageDetailsPatch()
np.SilenceLogs("Warning - streaming didn\'t find any chunks it could stream away...\n")

local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")

local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
local util = dofile_once("mods/quant.ew/files/src/util.lua")
local inventory_helper = dofile_once("mods/quant.ew/files/src/inventory_helper.lua")
local pretty = dofile_once("mods/quant.ew/files/lib/pretty_print.lua")
local perk_fns = dofile_once("mods/quant.ew/files/src/perk_fns.lua")

local version = dofile_once("mods/quant.ew/files/version.lua") or "unknown (dev build)"
print("Noita EW version: "..version)

dofile_once("data/scripts/lib/coroutines.lua")

ModLuaFileAppend("data/scripts/gun/gun.lua", "mods/quant.ew/files/append/gun.lua")
ModLuaFileAppend("data/scripts/gun/gun_actions.lua", "mods/quant.ew/files/append/action_fix.lua")

ModMagicNumbersFileAdd("mods/quant.ew/files/magic.xml")

local function load_modules()
    ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/item_sync.lua")
    
    ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/player_sync.lua")
    ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/enemy_sync.lua")
    ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/damage/sync.lua")
    ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/nickname.lua")

    if ctx.debug then
        ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/debug.lua")
    end
    
    ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/fungal_shift/sync.lua")
    ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/weather_sync.lua")
    ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/polymorph/sync.lua")

    if ctx.proxy_opt.world_sync_version == "1" then
        ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/world_sync_v1.lua")
    else
        ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/world_sync_v2.lua")
    end

    ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/heart_pickups/sync.lua")
    ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/spawn_hooks/init.lua")
    ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/proxy_info.lua")
    ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/perk_patches/init.lua")

    if ctx.proxy_opt.player_tether == "true" then
        ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/player_tether/player_tether.lua")
    end

    ctx.dofile_and_add_hooks("mods/quant.ew/files/src/system/kolmi/kolmi.lua")
end

function OnProjectileFired(shooter_id, projectile_id, initial_rng, position_x, position_y, target_x, target_y, send_message,
    unknown1, multicast_index, unknown3)
    ctx.hook.on_projectile_fired(shooter_id, projectile_id, initial_rng, position_x, position_y, target_x, target_y, send_message, unknown1, multicast_index, unknown3)
    if not EntityHasTag(shooter_id, "player_unit") and not EntityHasTag(shooter_id, "ew_client") then
        return -- Not fired by player, we don't care about it (for now?)
    end
    EntityAddTag(projectile_id, "ew_no_enemy_sync")
    local projectileComponent = EntityGetFirstComponentIncludingDisabled(projectile_id, "ProjectileComponent")
    local entity_that_shot    = ComponentGetValue2(projectileComponent, "mEntityThatShot")

    local shooter_player_data = player_fns.get_player_data_by_local_entity_id(shooter_id)
    local rng = 0
    -- Was shot locally
    if shooter_id == ctx.my_player.entity then
        -- If it was an initial shot by host
        if (entity_that_shot == 0 and multicast_index ~= -1 and unknown3 == 0) then
            rng = initial_rng
            table.insert(shooter_player_data.projectile_rng_init, rng)
        else
            rng = shooter_player_data.projectile_seed_chain[entity_that_shot] + 25
        end
    else
        if (entity_that_shot == 0 and multicast_index ~= -1 and unknown3 == 0) then
            if #shooter_player_data.projectile_rng_init > 0 then
                rng = table.remove(shooter_player_data.projectile_rng_init, 1)
            else
                -- Shouldn't happen
                -- GamePrint("No values in projectile_rng_init")
                rng = 0
            end
        else
            rng = (shooter_player_data.projectile_seed_chain[entity_that_shot] or 0) + 25
        end
    end
    shooter_player_data.projectile_seed_chain[projectile_id] = rng
    -- GamePrint("on fired "..projectile_id.." "..entity_that_shot.." "..shooter_id.." "..rng)
    np.SetProjectileSpreadRNG(rng)
end

function OnProjectileFiredPost(shooter_id, projectile_id, rng, position_x, position_y, target_x, target_y, send_message,
    unknown1, multicast_index, unknown3)
end

function OnPausedChanged(paused, is_wand_pickup)
    ctx.is_wand_pickup = is_wand_pickup
	local players = EntityGetWithTag("ew_current_player") or {}

	if (players[1]) then
		np.RegisterPlayerEntityId(players[1])
		local inventory_gui = EntityGetFirstComponentIncludingDisabled(players[1], "InventoryGuiComponent")
		local controls_component = EntityGetFirstComponentIncludingDisabled(players[1], "ControlsComponent")
		if (paused) then
			--EntitySetComponentIsEnabled(players[1], inventory_gui, false)
			np.EnableInventoryGuiUpdate(false)
			np.EnablePlayerItemPickUpper(false)
			ComponentSetValue2(controls_component, "enabled", false)
		else
			--EntitySetComponentIsEnabled(players[1], inventory_gui, true)
			np.EnableInventoryGuiUpdate(true)
			np.EnablePlayerItemPickUpper(true)
			ComponentSetValue2(controls_component, "enabled", true)
		end
	end
end

function OnWorldInitialized() -- This is called once the game world is initialized. Doesn't ensure any world chunks actually exist. Use OnPlayerSpawned to ensure the chunks around player have been loaded or created.
	--GamePrint( "OnWorldInitialized() " .. tostring(GameGetFrameNum()) )
end



function OnPlayerSpawned( player_entity ) -- This runs when player entity has been created
    print("Initial player entity: "..player_entity)
    
    if GlobalsGetValue("ew_player_count", "") == "" then
        GlobalsSetValue("ew_player_count", "1")
    end

    local x, y = EntityGetTransform(player_entity)
    ctx.initial_player_pos = {x=x, y=y}

    local my_player = player_fns.make_playerdata_for(player_entity, ctx.my_id)
    GamePrint("My peer_id: "..ctx.my_id)
    ctx.players[ctx.my_id] = my_player
    ctx.player_data_by_local_entity[player_entity] = my_player
    ctx.ready = true
    ctx.my_player = my_player

    if ctx.is_host then
        EntityAddTag(player_entity, "ew_host")
    else
        -- EntityAddComponent2(player_entity, "LuaComponent", {script_damage_about_to_be_received = "mods/quant.ew/files/cbs/immortal.lua"})
    end

    EntityAddTag(player_entity, "ew_current_player") -- TODO maybe remove and replace uses with player_unit tag
    EntityAddTag(player_entity, "ew_peer")

    EntityAddComponent2(player_entity, "LuaComponent", {script_wand_fired = "mods/quant.ew/files/cbs/count_times_wand_fired.lua"})

    net.send_welcome()

    local item_pick = EntityGetFirstComponentIncludingDisabled(player_entity, "ItemPickUpperComponent")
    ComponentSetValue2(item_pick, "is_immune_to_kicks", true)

    ctx.hook.on_local_player_spawn(my_player)
    ctx.hook.on_should_send_updates()

    GamePrint("Noita Entangled Worlds version "..version)

    OnPausedChanged(false, false)
    
    print("Game state entity: "..GameGetWorldStateEntity())
end

local function on_world_pre_update_inner()
    if ctx.my_player == nil then return end

    GlobalsSetValue("ew_player_rng", tostring(GameGetFrameNum()))

    net.update()

    local inventory_gui_comp = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "InventoryGuiComponent")
    if inventory_gui_comp and inventory_gui_comp ~= 0 then
        local inventory_open = ComponentGetValue2(inventory_gui_comp, "mActive")
        if ctx.is_inventory_open and not inventory_open then
            ctx.events.inventory_maybe_just_changed = true
        end
        ctx.is_inventory_open = inventory_open
    end

    if GameGetFrameNum() % 120 == 0 then
        player_fns.respawn_if_necessary()
        player_fns.spread_max_health()
    end

    if ctx.events.new_player_just_connected or ctx.events.inventory_maybe_just_changed or (GameGetFrameNum() % 5 == 0 and inventory_helper.has_inventory_changed(ctx.my_player)) then
        async(function()
            -- Wait 1 frame because apperently it takes some time for an item to get properly "registered" in an inventory?
            wait(1)
            local inventory_state = player_fns.serialize_items(ctx.my_player)
            if inventory_state ~= nil then
                -- GamePrint("Sending updated inventory")
                net.send_player_inventory(inventory_state)
            end
        end)
    end

    -- Perk sync
    if GameGetFrameNum() % 120 == 0 and not ctx.run_ended then
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

    wake_up_waiting_threads(1)
end

function OnWorldPreUpdate() -- This is called every time the game is about to start updating the world
    util.tpcall(on_world_pre_update_inner)
end

local function on_world_post_update_inner()
    if ctx.my_player == nil then return end

    if not ctx.run_ended then
        ctx.hook.on_world_update_post()
    end

    local times_wand_fired = tonumber(GlobalsGetValue("ew_wand_fired", "0"))
    GlobalsSetValue("ew_wand_fired", "0")
    if times_wand_fired > 0 then
        local inventory_component = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "Inventory2Component")
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

function OnWorldPostUpdate() -- This is called every time the game has finished updating the world
	util.tpcall(on_world_post_update_inner)
    ctx.events = {}
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
    load_modules()
end

function OnModInit() end


function OnModPostInit() end

print("entangled_worlds init ok")