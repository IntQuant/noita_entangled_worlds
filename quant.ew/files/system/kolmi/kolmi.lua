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

rpc.opts_reliable()
function rpc.spawn_kolmi(gid)
    if not GameHasFlagRun("ew_sampo_picked") then
        local item_id = ewext.find_by_gid(gid)
        if item_id ~= nil and util.do_i_own(item_id) then
            GameAddFlagRun("ew_sampo_picked")
            dofile("data/entities/animals/boss_centipede/sampo_pickup.lua")
            item_pickup(item_id, nil, nil, true)
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

util.add_cross_call("ew_spawn_kolmi", rpc.spawn_kolmi)

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
