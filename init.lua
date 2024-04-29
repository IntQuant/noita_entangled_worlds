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
	GamePrint( "OnWorldInitialized() " .. tostring(GameGetFrameNum()) )
    ctx.init()
end

local my_player = nil
local other_player = nil

function OnPlayerSpawned( player_entity ) -- This runs when player entity has been created
	GamePrint( "OnPlayerSpawned() - Player entity id: " .. tostring(player_entity) )
    local x, y = EntityGetFirstHitboxCenter(player_entity)
    -- local other = EntityLoad("mods/quant.ew/files/entities/client.xml", x, y)
    -- np.SetPlayerEntity(player_entity)

    my_player = player_fns.make_playerdata_for(player_entity)
    ctx.players[ctx.host_id] = my_player
    -- other_player = player_fns.make_playerdata_for(other)
end

function OnWorldPreUpdate() -- This is called every time the game is about to start updating the world
    if my_player == nil then return end
	GamePrint( "Pre-update hook " .. tostring(GameGetFrameNum()) )
    
    local events = net.get_events()

    for i=1,#events do
        local event = events[i]
        GamePrint("event "..i.." "..event.kind)
        if event.kind == "connect" then
            player_fns.spawn_player_for(event.peer_id)
        end
    end

    -- local sinp = player_fns.serialize_inputs(my_player)
    -- player_fns.deserialize_inputs(sinp, other_player)
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
    if ctx.players ~= nil then
        print("ctx.players is not nil")
    else
        print("ctx.players is nil")
    end
end

function OnModPostInit()
	
end

print("entangled_worlds init ok")