dofile_once("data/scripts/lib/coroutines.lua")

ModLuaFileAppend("data/scripts/biomes/boss_arena.lua", "mods/quant.ew/files/system/kolmi/append/boss_arena.lua")
ModLuaFileAppend(
    "data/entities/animals/boss_centipede/sampo_pickup.lua",
    "mods/quant.ew/files/system/kolmi/append/spawn_kolmi.lua"
)
ModLuaFileAppend(
    "data/entities/animals/boss_centipede/boss_centipede_update.lua",
    "mods/quant.ew/files/system/kolmi/append/boss_update.lua"
)
util.replace_text_in(
    "data/entities/animals/boss_centipede/boss_centipede_before_fight.lua",
    [[local player_nearby = false]],
    [[local player_nearby = #EntityGetInRadiusWithTag(x, y, 128, "ew_peer") > 0]]
)

local rpc = net.new_rpc_namespace()

local module = {}

rpc.opts_reliable()
function rpc.spawn_portal(x, y)
    EntityLoad("data/entities/buildings/teleport_ending_victory_delay.xml", x, y)
end

local function animate_sprite(current_name, next_name)
    local kolmi = EntityGetClosestWithTag(0, 0, "boss_centipede")
    if kolmi ~= nil and kolmi ~= 0 then
        GamePlayAnimation(kolmi, current_name, 0, next_name, 0)
    end
end

rpc.opts_reliable()
function rpc.kolmi_anim(current_name, next_name, is_aggro)
    if not is_aggro then
        animate_sprite(current_name, next_name)
    else
        -- aggro overrides animations
        animate_sprite("aggro", "aggro")
    end
end

local function switch_shield(entity_id, is_on)
    local children = EntityGetAllChildren(entity_id)
    if children == nil then
        return
    end
    for _, v in ipairs(children) do
        if EntityGetName(v) == "shield_entity" then
            if is_on then
                EntitySetComponentsWithTagEnabled(v, "shield", true)
                -- muzzle flash
                local x, y = EntityGetTransform(entity_id)
                EntityLoad("data/entities/particles/muzzle_flashes/muzzle_flash_circular_large_pink_reverse.xml", x, y)
                GameEntityPlaySound(v, "activate")
                return true
            else
                EntitySetComponentsWithTagEnabled(v, "shield", false)
                -- muzzle flash
                local x, y = EntityGetTransform(entity_id)
                EntityLoad("data/entities/particles/muzzle_flashes/muzzle_flash_circular_large_pink.xml", x, y)
                GameEntityPlaySound(v, "deactivate")
                return true
            end
        end
    end
end

rpc.opts_reliable()
function rpc.kolmi_shield(is_on, orbcount)
    local kolmi = EntityGetClosestWithTag(0, 0, "boss_centipede")
    if kolmi == nil or kolmi == 0 then
        return
    end

    if switch_shield(kolmi, is_on) then
        return
    end

    -- No shield?
    local pos_x, pos_y = EntityGetTransform(kolmi)
    if orbcount == 0 then
        EntityAddChild(
            kolmi,
            EntityLoad("data/entities/animals/boss_centipede/boss_centipede_shield_weak.xml", pos_x, pos_y)
        )
    else
        EntityAddChild(
            kolmi,
            EntityLoad("data/entities/animals/boss_centipede/boss_centipede_shield_strong.xml", pos_x, pos_y)
        )
    end
    switch_shield(kolmi, is_on)
end

-- The pickup's sound, music and sparkle from the base sampo_pickup.lua.
local function play_pickup(x, y)
    GameTriggerMusicFadeOutAndDequeueAll(10.0)
    GamePlaySound("data/audio/Desktop/event_cues.bank", "event_cues/sampo_pick/create", x, y)
    GameTriggerMusicEvent("music/boss_arena/battle", false, x, y)
    EntityLoad("data/entities/particles/image_emitters/chest_effect.xml", x, y)
end

-- A pickup as it happens. Every peer plays it once and remembers it; starting the
-- fight is up to whoever owns Kolmi, in module.on_world_update below.
rpc.opts_reliable()
rpc.opts_everywhere()
function rpc.sampo_picked(x, y)
    if GameHasFlagRun("ew_sampo_picked") then
        return
    end
    GameAddFlagRun("ew_sampo_picked")
    play_pickup(x, y)
end

-- For a peer that wasn't connected when the sampo was picked up. Only remembered,
-- not played: the moment has passed.
rpc.opts_reliable()
function rpc.sampo_was_picked()
    GameAddFlagRun("ew_sampo_picked")
end

-- Runs whenever a peer connects, so a late joiner - who may go on to own Kolmi -
-- hears about the pickup from anyone who saw it. The flag survives a reload, so a
-- reloaded peer keeps passing it on too.
function module.on_should_send_updates()
    if GameHasFlagRun("ew_sampo_picked") then
        rpc.sampo_was_picked()
    end
end

-- Only a Kolmi entity sync has given to this peer. An untracked one has no owner
-- yet, and a remote copy is overwritten by its owner every frame.
local function is_mine(ent)
    for _, v in ipairs(EntityGetComponentIncludingDisabled(ent, "VariableStorageComponent") or {}) do
        if ComponentGetValue2(v, "name") == "ew_gid_lid" then
            return ComponentGetValue2(v, "value_bool")
        end
    end
    return false
end

-- The fight half of the base sampo_pickup.lua, for Kolmi alone - rpc.sampo_picked
-- already played the rest. For a Kolmi whose fight had started before it changed
-- owner, enable_later in ewext switches the same components back on instead.
local function start_fight(kolmi, reference)
    local x, y = EntityGetTransform(reference)
    -- Only on the owner: its boss_centipede_update.lua is what sets it back to 0.
    GlobalsSetValue("FINAL_BOSS_ACTIVE", "1")
    EntitySetComponentsWithTagEnabled(kolmi, "disabled_at_start", true)
    EntitySetComponentsWithTagEnabled(kolmi, "enabled_at_start", false)
    PhysicsSetStatic(kolmi, false)
    EntityAddTag(kolmi, "boss_centipede_active")
    for _, child in ipairs(EntityGetAllChildren(kolmi) or {}) do
        if EntityHasTag(child, "protection") then
            EntityKill(child)
        end
    end
    EntityLoad("data/entities/animals/boss_centipede/loose_lavaceiling.xml", x - 235, y - 73)
    EntityLoad("data/entities/animals/boss_centipede/loose_lavaceiling.xml", x + 264, y - 50)
    EntityLoad("data/entities/animals/boss_centipede/loose_lavabridge.xml", x - 235, y + 282)
    EntityLoad("data/entities/animals/boss_centipede/loose_lavabridge.xml", x + 257, y + 262)
end

-- Kolmi's owner is the only peer whose Kolmi really fights; entity sync starts
-- every other copy once the owner reports the fight has begun. Checked every
-- frame rather than once when the pickup arrives, so a Kolmi that changes owner
-- before its fight starts is started by the new owner instead.
function module.on_world_update()
    if not GameHasFlagRun("ew_sampo_picked") then
        return
    end
    -- The lava pieces are placed relative to the arena's reference point.
    local reference = EntityGetWithTag("reference")[1]
    if reference == nil then
        return
    end
    for _, kolmi in ipairs(EntityGetWithTag("boss_centipede")) do
        if not EntityHasTag(kolmi, "boss_centipede_active") and is_mine(kolmi) then
            start_fight(kolmi, reference)
            local newgame_n = tonumber(SessionNumbersGetValue("NEW_GAME_PLUS_COUNT"))
            local orbcount = GameGetOrbCountThisRun() + newgame_n
            rpc.kolmi_shield(true, orbcount)
        end
    end
end

--[[util.add_cross_call("ew_sampo_spawned", function()
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
        --ctx.cap.item_sync.globalize(sampo_ent)
    else
        EntityKill(sampo_ent)
    end
end)]]

util.add_cross_call("ew_kolmi_spawn_portal", rpc.spawn_portal)

util.add_cross_call("ew_kolmi_anim", rpc.kolmi_anim)

util.add_cross_call("ew_kolmi_shield", rpc.kolmi_shield)

util.add_cross_call("ew_sampo_picked", rpc.sampo_picked)

--[[ctx.cap.item_sync.register_pickup_handler(function(item_id)
    if ctx.is_host and EntityHasTag(item_id, "this_is_sampo") then
        -- Check if it's the first time we pick it up to avoid that sound on later pickups.
        if not GameHasFlagRun("ew_sampo_picked") then
            GameAddFlagRun("ew_sampo_picked")
            dofile("data/entities/animals/boss_centipede/sampo_pickup.lua")
            item_pickup(item_id)
            local newgame_n = tonumber(SessionNumbersGetValue("NEW_GAME_PLUS_COUNT"))
            local orbcount = GameGetOrbCountThisRun() + newgame_n
            rpc.kolmi_shield(true, orbcount)
        end
    end
end)]]

return module
