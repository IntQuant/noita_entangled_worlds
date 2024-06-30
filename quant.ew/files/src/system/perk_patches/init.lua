local util = dofile_once("mods/quant.ew/files/src/util.lua")
local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local net = dofile_once("mods/quant.ew/files/src/net.lua")
local player_fns = dofile_once("mods/quant.ew/files/src/player_fns.lua")
local np = require("noitapatcher")

local rpc = net.new_rpc_namespace()

local module = {}

ModLuaFileAppend("data/scripts/perks/perk_list.lua", "mods/quant.ew/files/src/system/perk_patches/append/perk_list.lua")

rpc.opts_reliable()
rpc.opts_everywhere()
function rpc.modify_max_hp(percent_amount, do_heal)
    if ctx.is_host then
        local player_count = tonumber(GlobalsGetValue("ew_player_count", "1"))
        local health = ctx.cap.health
        local max_hp = health.max_health()
        health.set_max_health(max_hp + max_hp / player_count * (percent_amount-1))
        if do_heal then
            local hp = health.health()
            health.set_health(hp + max_hp / player_count * (percent_amount-1))
        end
        if health.health() > health.max_health() then
            health.set_health(health.max_health())
        end
    end
end

np.CrossCallAdd("ew_perks_modify_max_hp", rpc.modify_max_hp)

return module
