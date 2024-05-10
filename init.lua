dofile_once("mods/quant.ew/NoitaPatcher/load.lua")
local np = require("noitapatcher")

dofile_once( "data/scripts/lib/utilities.lua" )

np.InstallShootProjectileFiredCallbacks()
np.EnableGameSimulatePausing(false)
np.InstallDamageDetailsPatch()
np.SilenceLogs("Warning - streaming didn\'t find any chunks it could stream away...\n")

local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local pretty = dofile_once("mods/quant.ew/files/lib/pretty_print.lua")
local perk_fns = dofile_once("mods/quant.ew/files/src/perk_fns.lua")
local enemy_sync = dofile_once("mods/quant.ew/files/src/enemy_sync.lua")
local world_sync = dofile_once("mods/quant.ew/files/src/world_sync.lua")

function OnProjectileFired(shooter_id, projectile_id, rng, position_x, position_y, target_x, target_y, send_message,
    unknown1, multicast_index, unknown3)
end

function OnProjectileFiredPost(shooter_id, projectile_id, rng, position_x, position_y, target_x, target_y, send_message,
    unknown1, multicast_index, unknown3)
end

function OnPausedChanged(paused, is_wand_pickup)
	local players = EntityGetWithTag("player_unit") or {}

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

-- all functions below are optional and can be left out

--[[

function OnModPreInit()
	print("Mod - OnModPreInit()") -- First this is called for all mods
end

function OnModInit()
	print("Mod - OnModInit()") -- After that this is called for all mods
end

function OnModPostInit()
	print("Mod - OnModPostInit()") -- Then this is called for all mods
end

function OnWorldPostUpdate() -- This is called every time the game has finished updating the world
	GamePrint( "Post-update hook " .. tostring(GameGetFrameNum()) )
end

]]--
function OnWorldInitialized() -- This is called once the game world is initialized. Doesn't ensure any world chunks actually exist. Use OnPlayerSpawned to ensure the chunks around player have been loaded or created.
	--GamePrint( "OnWorldInitialized() " .. tostring(GameGetFrameNum()) )
end

local my_player = nil

function OnPlayerSpawned( player_entity ) -- This runs when player entity has been created
	GamePrint( "OnPlayerSpawned() - Player entity id: " .. tostring(player_entity) )

    my_player = player_fns.make_playerdata_for(player_entity, ctx.my_id)
    GamePrint("My peer_id: "..ctx.my_id)
    ctx.players[ctx.my_id] = my_player
    ctx.ready = true
    ctx.is_host = ctx.my_id == ctx.host_id

    np.SetPauseState(4)
    np.SetPauseState(0)

    dofile_once("data/scripts/perks/perk.lua")
    local x, y = EntityGetFirstHitboxCenter(player_entity)
    perk_spawn(x, y, "LASER_AIM", true)
    perk_spawn(x-50, y, "GLASS_CANNON", true)
end

function OnWorldPreUpdate() -- This is called every time the game is about to start updating the world
    local result, err = pcall(on_world_pre_update_inner)
    if not result then
        GamePrint(tostring(err))
    end
end
function on_world_pre_update_inner()
    if my_player == nil then return end
	-- GamePrint( "Pre-update hook " .. tostring(GameGetFrameNum()) )
    
    net.update()
    if GameGetFrameNum() % 1 == 0 then
        local input_data = player_fns.serialize_inputs(my_player)
        local pos_data =  player_fns.serialize_position(my_player)
        local current_slot = player_fns.get_current_slot(my_player)
        net.send_player_update(input_data, pos_data, current_slot)
    end

    if GameGetFrameNum() % 2 == 1 then
        if ctx.is_host then
            net.send_enemy_data(enemy_sync.host_upload_entities())
        else
            enemy_sync.client_cleanup()
        end
    end

    if ctx.is_host then
        local world_data = world_sync.host_upload()
        if world_data ~= nil then
            net.send_world_data(world_data)
        end
    end

    if GameGetFrameNum() % 120 == 0 then
        -- GamePrint("Serialize items")
        local inventory_state = player_fns.serialize_items(my_player)
        net.send_player_inventory(inventory_state)
        local perk_data = perk_fns.get_my_perks()
        -- print(pretty.table(perk_data))
        net.send_player_perks(perk_data)
    end
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
end

function OnModPostInit()
	
end

print("entangled_worlds init ok")