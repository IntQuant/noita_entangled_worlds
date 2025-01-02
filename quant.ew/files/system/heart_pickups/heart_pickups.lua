ModLuaFileAppend("data/scripts/items/heart.lua", "mods/quant.ew/files/system/heart_pickups/append/heart.lua")
ModLuaFileAppend(
    "data/scripts/items/heart_better.lua",
    "mods/quant.ew/files/system/heart_pickups/append/heart_better.lua"
)
ModLuaFileAppend("data/scripts/items/heart_evil.lua", "mods/quant.ew/files/system/heart_pickups/append/heart_evil.lua")
ModLuaFileAppend(
    "data/scripts/items/heart_fullhp.lua",
    "mods/quant.ew/files/system/heart_pickups/append/heart_fullhp.lua"
)
ModLuaFileAppend(
    "data/scripts/items/heart_fullhp_temple.lua",
    "mods/quant.ew/files/system/heart_pickups/append/heart_fullhp_temple.lua"
)

local rpc = net.new_rpc_namespace()
local module = {}

local function heart_pickup(heart)
    local multiplier = tonumber(GlobalsGetValue("HEARTS_MORE_EXTRA_HP_MULTIPLIER", "1"))
    local do_heal_table = {
        fullhp = true,
        temple = true,
    }
    local max_hp_increase_table = {
        normal = { 1, false },
        better = { 2, false },
        evil = { 2, false },
        temple = { 0.4, true },
    }
    local do_heal = do_heal_table[heart] or false
    local max_hp_increase = max_hp_increase_table[heart]

    local hp, max_hp = ctx.cap.health.health(), ctx.cap.health.max_health()
    --local cap = util.get_ent_health_cap(ctx.my_player.entity)
    local player_count = tonumber(GlobalsGetValue("ew_player_count", "1"))

    local max_hp_old = max_hp

    if max_hp_increase ~= nil then
        max_hp = max_hp + max_hp_increase[1] * multiplier
        local adjust_cap = max_hp_increase[2]
        if adjust_cap then
            util.set_ent_health_cap(max_hp)
        end
    end

    if do_heal then
        local healing = math.min(max_hp - hp, max_hp / player_count)
        hp = hp + healing
    end

    ctx.cap.health.set_max_health(max_hp)
    ctx.cap.health.set_health(hp)
    -- util.set_ent_health(ctx.my_player.entity, {hp, max_hp})

    local peer_data = ctx.rpc_player_data or ctx.my_player
    if max_hp ~= max_hp_old and heart == "evil" then
        local entity_who_picked = peer_data.entity
        local x_pos, y_pos = EntityGetTransform(entity_who_picked)
        local child_id = EntityLoad("data/entities/misc/effect_poison_big.xml", x_pos, y_pos)
        EntityAddChild(entity_who_picked, child_id)
    end

    GamePrint("Player " .. peer_data.name .. " picked up a heart")
    GameTriggerMusicCue("item")
end

function module.on_world_update()
    local heart = GlobalsGetValue("ew_heart_pickup", "")
    if heart ~= "" then
        rpc.heart_pickup(heart)
        heart_pickup(heart)
        GlobalsSetValue("ew_heart_pickup", "")
    end
end

rpc.opts_reliable()
function rpc.heart_pickup(heart)
    heart_pickup(heart)
end

return module
