local util = dofile_once("mods/quant.ew/files/src/util.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local np = require("noitapatcher")

ModLuaFileAppend("data/scripts/biomes/boss_arena.lua", "mods/quant.ew/files/src/system/kolmi/append/boss_arena.lua")
ModLuaFileAppend("data/entities/animals/boss_centipede/boss_centipede_update.lua", "mods/quant.ew/files/src/system/kolmi/append/boss_update.lua")
util.replace_text_in("data/entities/animals/boss_centipede/boss_centipede_before_fight.lua",
    [[local player_nearby = false]], [[local player_nearby = #EntityGetInRadiusWithTag(x, y, 128, "ew_peer") > 0]])

local rpc = net.new_rpc_namespace()

local module = {}

rpc.opts_reliable()
function rpc.spawn_portal(x, y)
    EntityLoad( "data/entities/buildings/teleport_ending_victory_delay.xml", x, y )
end

local function animate_sprite( current_name, next_name )
	local kolmi = EntityGetClosestWithTag(0, 0, "boss_centipede")
    if kolmi ~= nil and kolmi ~= 0 then
        GamePlayAnimation( kolmi, current_name, 0, next_name, 0 )
    end
end

rpc.opts_reliable()
function rpc.kolmi_anim(current_name, next_name, is_aggro)
    if not is_aggro then
        animate_sprite( current_name, next_name )
    else
        -- aggro overrides animations
        animate_sprite( "aggro", "aggro" )
    end
end

np.CrossCallAdd("ew_sampo_spawned", function()
    local sampo_ent = EntityGetClosestWithTag(0, 0, "this_is_sampo")
    if sampo_ent == nil or sampo_ent == 0 then
        -- In case sampo wasn't actually spawned.
        return
    end
    if ctx.is_host then
        -- First lua component is the one that has pickup script.
        local pickup_component = EntityGetFirstComponentIncludingDisabled(sampo_ent, "LuaComponent")
        -- Remove it as to not handle pickup twice.
        EntityRemoveComponent(sampo_ent, pickup_component)
        ctx.cap.item_sync.globalize(sampo_ent)
    else
        EntityKill(sampo_ent)
    end
end)

np.CrossCallAdd("ew_kolmi_spawn_portal", rpc.spawn_portal)

np.CrossCallAdd("ew_kolmi_anim", rpc.kolmi_anim)

ctx.cap.item_sync.register_pickup_handler(function(item_id)
    if ctx.is_host and EntityHasTag(item_id, "this_is_sampo") then
        -- Check if it's the first time we pick it up to avoid that sound on later pickups.
        if not GameHasFlagRun("ew_sampo_picked") then
            GameAddFlagRun("ew_sampo_picked")
            dofile("data/entities/animals/boss_centipede/sampo_pickup.lua")
            item_pickup(item_id)
        end
    end
end)

return module