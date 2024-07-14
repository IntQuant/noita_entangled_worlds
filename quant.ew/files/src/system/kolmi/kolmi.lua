local util = dofile_once("mods/quant.ew/files/src/util.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local np = require("noitapatcher")

ModLuaFileAppend("data/scripts/biomes/boss_arena.lua", "mods/quant.ew/files/src/system/kolmi/append/boss_arena.lua")

local rpc = net.new_rpc_namespace()

local module = {}


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

ctx.cap.item_sync.register_pickup_handler(function(item_id)
    if ctx.is_host and EntityHasTag(item_id, "this_is_sampo") then
        dofile("data/entities/animals/boss_centipede/sampo_pickup.lua")
        item_pickup(item_id)
    end
end)

return module