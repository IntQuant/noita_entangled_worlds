local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")
local np = require("noitapatcher")
local perk_fns = dofile_once("mods/quant.ew/files/core/perk_fns.lua")

local rpc = net.new_rpc_namespace()

local module = {}

local default_items = 5

ModLuaFileAppend("data/scripts/perks/perk_list.lua", "mods/quant.ew/files/system/perk_patches/append/perks_common.lua")
ModLuaFileAppend("data/scripts/perks/perk_utilities.lua", "mods/quant.ew/files/system/perk_patches/append/cosmetics_append.lua")

if ctx.proxy_opt.game_mode == "shared_health" then
    print("Loading extra perk patches for shared health mode")
    ModLuaFileAppend("data/scripts/perks/perk_list.lua", "mods/quant.ew/files/system/perk_patches/append/perks_shared.lua")
else
    print("Loading extra perk patches for local health mode")
    ModLuaFileAppend("data/scripts/perks/perk_list.lua", "mods/quant.ew/files/system/perk_patches/append/perks_local.lua")
end

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

rpc.opts_everywhere()
function rpc.sync_perk_amount(items, genome)
    GlobalsSetValue("TEMPLE_SHOP_ITEM_COUNT", tostring(items))
    local world = GameGetWorldStateEntity()
    local com = EntityGetFirstComponent(world, "WorldStateComponent")
    ComponentSetValue(com, "global_genome_relations_modifier", tostring(genome))
end

function module.on_world_update()
    if GameGetFrameNum() == 5 then
        default_items = tonumber(GlobalsGetValue("TEMPLE_SHOP_ITEM_COUNT", "5"))
    end
    if ctx.is_host and GameGetFrameNum() % 120 == 24 then
        local items = default_items
        local genome = 0
        for peer, player in pairs(ctx.players) do
            local perks = {}
            if peer == ctx.my_id then
                perks = perk_fns.get_my_perks()
            else
                perks = util.get_ent_variable(player.entity, "ew_current_perks") or {}
            end
            for perk, count in pairs(perks) do
                if perk == "EXTRA_SHOP_ITEM" then
                    items = items + count
                elseif perk == "GENOME_MORE_LOVE" then
                    genome = genome + count
                elseif perk == "GENOME_MORE_HATRED" then
                    genome = genome - count
                end
            end
        end
        rpc.sync_perk_amount(items, genome * 25)
    end
end

return module