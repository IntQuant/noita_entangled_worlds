dofile_once("mods/quant.ew/NoitaPatcher/load.lua")
np = require("noitapatcher")

package.cpath = package.cpath .. ";./mods/quant.ew/?.dll"
package.path = package.path .. ";./mods/quant.ew/?.lua"
print(package.cpath)

dofile_once("data/scripts/lib/utilities.lua")

np.InstallShootProjectileFiredCallbacks()
np.EnableGameSimulatePausing(false)
np.InstallDamageDetailsPatch()
np.SilenceLogs("Warning - streaming didn't find any chunks it could stream away...\n")

ewext = require("ewext1")

-- Make some stuff global, as it's way too annoying to import each time.
constants = dofile_once("mods/quant.ew/files/core/constants.lua")
ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
util = dofile_once("mods/quant.ew/files/core/util.lua")
net = dofile_once("mods/quant.ew/files/core/net.lua")
inventory_helper = dofile_once("mods/quant.ew/files/core/inventory_helper.lua")
player_fns = dofile_once("mods/quant.ew/files/core/player_fns.lua")

local cos = dofile_once("mods/quant.ew/files/system/player/player_cosmetics.lua")

local perk_fns = dofile_once("mods/quant.ew/files/core/perk_fns.lua")

local version = ModDoesFileExist("mods/quant.ew/files/version.lua") and dofile_once("mods/quant.ew/files/version.lua")
    or "unknown (dev build)"
print("Noita EW version: " .. version)

dofile_once("data/scripts/lib/coroutines.lua")

ModLuaFileAppend("data/scripts/gun/gun.lua", "mods/quant.ew/files/resource/append/gun.lua")
ModLuaFileAppend("data/scripts/gun/gun_actions.lua", "mods/quant.ew/files/resource/append/action_fix.lua")

ModMagicNumbersFileAdd("mods/quant.ew/files/magic.xml")

util.add_cross_call("ew_per_peer_seed", function()
    return tonumber(string.sub(ctx.my_id, 8, 12), 16), tonumber(string.sub(ctx.my_id, 12), 16)
end)

util.add_cross_call("ew_spectator", function()
    if ctx.spectating_over_peer_id == nil then
        return (ctx.my_player ~= nil and ctx.my_player.entity) or EntityGetWithTag("player_unit")[1]
    else
        return ctx.players[ctx.spectating_over_peer_id].entity
    end
end)

if GameHasFlagRun("ending_game_completed") then
    ModTextFileSetContent("data/entities/misc/loose_chunks.xml", "<Entity/>")
    ModTextFileSetContent("data/entities/misc/loose_chunks_huge.xml", "<Entity/>")
    ModTextFileSetContent("data/entities/projectiles/deck/crumbling_earth_effect.xml", "<Entity/>")
end

local function load_modules()
    ctx.load_system("uniq_flags")
    ctx.load_system("ewext_init")

    -- ctx.dofile_and_add_hooks("mods/quant.ew/files/system/item_sync.lua")

    ctx.dofile_and_add_hooks("mods/quant.ew/files/system/player_sync.lua")
    -- ctx.dofile_and_add_hooks("mods/quant.ew/files/system/enemy_sync.lua")

    if ctx.proxy_opt.game_mode == "shared_health" then
        ctx.load_system("damage")
        ctx.load_system("heart_pickups")
        ctx.load_system("patch_meat_biome")
        ctx.load_system("kivi_patch")
    end
    if ctx.proxy_opt.game_mode == "local_health" then
        ctx.load_system("local_health")
        ctx.load_system("notplayer_ai")
        ctx.load_system("spectator_helps")
        ctx.load_system("end_fight")
    end

    ctx.dofile_and_add_hooks("mods/quant.ew/files/system/nickname.lua")

    if ctx.proxy_opt.debug then
        ctx.dofile_and_add_hooks("mods/quant.ew/files/system/debug.lua")
    end

    ctx.load_system("fungal_shift")
    ctx.dofile_and_add_hooks("mods/quant.ew/files/system/weather_sync.lua")
    ctx.load_system("polymorph")

    ctx.load_system("world_sync")

    -- ctx.load_system("spawn_hooks")
    ctx.dofile_and_add_hooks("mods/quant.ew/files/system/proxy_info.lua")
    ctx.load_system("perk_patches")

    ctx.load_system("player_tether")

    ctx.load_system("kolmi")
    ctx.load_system("ending")
    ctx.load_system("spell_patches")
    ctx.load_system("enemy_scaling")

    ctx.load_system("patch_dragon_boss")

    ctx.load_system("player_arrows")
    ctx.load_system("player_ping")
    ctx.load_system("extra_genomes")
    ctx.load_system("game_effect_sync")
    ctx.load_system("orb_sync")
    ctx.load_system("flag_sync")
    ctx.load_system("essence_sync")
    ctx.load_system("spectate")
    ctx.load_system("effect_data_sync")
    -- if ctx.proxy_opt.item_dedup then
    -- ctx.load_system("gen_sync")
    -- end
    ctx.load_system("karl")
    ctx.load_system("remove_wand_sound")
    if ctx.proxy_opt.randomize_perks then
        ctx.load_system("randomize_perks")
    end
    ctx.load_system("streaming_sync")
    if not ctx.proxy_opt.same_loadout then
        ctx.load_system("rnd_wands")
    end
    ctx.load_system("worms")
    ctx.load_system("wand_charm")
    ctx.load_system("stevari")
    ctx.load_system("angry_ghost_memory")
    ctx.load_system("gate_boss")
    ctx.load_system("tapion")
    ctx.load_system("world_sync_cuts")
    ctx.load_system("hamis")
    ctx.load_system("greed")
    ctx.load_system("spell_refresh")
    ctx.load_system("shiny_orb")
    ctx.load_system("potion_mimic")
    ctx.load_system("map")
    ctx.load_system("homunculus")
    ctx.load_system("text")
    ctx.load_system("ragdoll_fix")
    ctx.load_system("explosion_cuts")
    ctx.load_system("wang_hooks")
    ctx.load_system("entity_sync_helper")
    ctx.load_system("telekenisis")
    ctx.load_system("dice")
    ctx.load_system("sampo")
    ctx.load_system("meat")
    ctx.load_system("boss_damage")
    if ctx.proxy_opt.pvp then
        ctx.load_system("pvp")
    end
end

local function load_extra_modules()
    print("Starting to load extra stuff")
    for _, file in ipairs(ModLuaFileGetAppends("mods/quant.ew/files/api/extra_modules.lua")) do
        ctx.dofile_and_add_hooks(file)
    end
end

local last_mana = -1

local function fire()
    local inventory_component = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "Inventory2Component")
    if inventory_component ~= nil then
        local last_switch = ComponentGetValue2(inventory_component, "mLastItemSwitchFrame")
        local switched_now = last_switch == GameGetFrameNum()
        local special_seed = tonumber(GlobalsGetValue("ew_player_rng", "0"))
        local fire_data = player_fns.make_fire_data(special_seed, ctx.my_player, last_mana)
        if fire_data ~= nil then
            if switched_now then
                fire_data.switched_now = true
            end
            net.send_fire(fire_data)
        end
    end
end

function OnProjectileFired(
    shooter_id,
    projectile_id,
    initial_rng,
    position_x,
    position_y,
    target_x,
    target_y,
    send_message,
    unknown1,
    multicast_index,
    unknown3
)
    ctx.hook.on_projectile_fired(
        shooter_id,
        projectile_id,
        initial_rng,
        position_x,
        position_y,
        target_x,
        target_y,
        send_message,
        unknown1,
        multicast_index,
        unknown3
    )
    if not EntityHasTag(shooter_id, "player_unit") and not EntityHasTag(shooter_id, "ew_client") then
        return -- Not fired by player, we don't care about it (for now?)
    end
    EntityAddTag(projectile_id, "ew_no_enemy_sync")
    local projectileComponent = EntityGetFirstComponentIncludingDisabled(projectile_id, "ProjectileComponent")
    local entity_that_shot
    if projectileComponent == nil then
        entity_that_shot = GameGetWorldStateEntity()
    else
        entity_that_shot = ComponentGetValue2(projectileComponent, "mEntityThatShot")
    end

    local shooter_player_data = player_fns.get_player_data_by_local_entity_id(shooter_id)
    local rng = 0
    -- Was shot locally
    if shooter_id == ctx.my_player.entity then
        -- If it was an initial shot by host
        if entity_that_shot == 0 and multicast_index ~= -1 and unknown3 == 0 then
            if tonumber(GlobalsGetValue("ew_wand_fired", "0")) ~= 0 then
                rng = initial_rng
                table.insert(shooter_player_data.projectile_rng_init, rng)
            else
                rng = (shooter_player_data.projectile_seed_chain[shooter_id - 1] or 0) + 25
            end
        else
            rng = (shooter_player_data.projectile_seed_chain[entity_that_shot] or 0) + 25
        end
    else
        if entity_that_shot == 0 and multicast_index ~= -1 and unknown3 == 0 then
            if #shooter_player_data.projectile_rng_init > 0 then
                rng = table.remove(shooter_player_data.projectile_rng_init, 1)
            else
                rng = (shooter_player_data.projectile_seed_chain[shooter_id - 1] or 0) + 25
            end
        else
            rng = (shooter_player_data.projectile_seed_chain[entity_that_shot] or 0) + 25
        end
    end
    shooter_player_data.projectile_seed_chain[shooter_id - 1] = rng
    shooter_player_data.projectile_seed_chain[entity_that_shot] = rng
    shooter_player_data.projectile_seed_chain[projectile_id] = rng
    for _, lua in ipairs(EntityGetComponent(projectile_id, "LuaComponent") or {}) do
        local src = ComponentGetValue2(lua, "script_source_file")
        if
            src == "data/scripts/projectiles/transmutation.lua"
            or src == "data/scripts/projectiles/random_explosion.lua"
            or src == "data/scripts/projectiles/fizzle.lua"
            or src == "data/scripts/projectiles/teleport_cast.lua"
        then
            EntityAddComponent2(
                projectile_id,
                "VariableStorageComponent",
                { name = "ew_transmutation", value_int = rng }
            )
        end
    end
    local n = EntityGetFilename(projectile_id)
    if
        n == "data/entities/items/pickup/egg_hollow.xml"
        or string.sub(n, 1, 31) == "data/entities/items/pickup/egg_"
    then
        EntityAddComponent2(projectile_id, "VariableStorageComponent", { _tags = "ew_egg", value_int = rng })
        local body = EntityGetFirstComponentIncludingDisabled(projectile_id, "PhysicsBody2Component")
        if body ~= nil then
            ComponentSetValue2(body, "destroy_body_if_entity_destroyed", true)
        end
        if shooter_player_data.peer_id ~= ctx.my_id and string.sub(n, 1, 31) == "data/entities/items/pickup/egg_" then
            local exp = EntityGetFirstComponentIncludingDisabled(projectile_id, "ExplodeOnDamageComponent")
            if exp ~= nil then
                ComponentObjectSetValue2(exp, "config_explosion", "load_this_entity", "")
            end
        end
        --ewext.sync_projectile(projectile_id, shooter_player_data.peer_id, rng)
    elseif EntityHasTag(projectile_id, "ew_projectile_position_sync") then
        local body = EntityGetFirstComponentIncludingDisabled(projectile_id, "PhysicsBody2Component")
        local proj = EntityGetFirstComponentIncludingDisabled(projectile_id, "ProjectileComponent")
        local life = EntityGetFirstComponentIncludingDisabled(projectile_id, "LifetimeComponent")
        if proj == nil or ComponentGetValue2(proj, "lifetime") > 4 or ComponentGetValue2(life, "lifetime") > 4 then
            ewext.sync_projectile(projectile_id, shooter_player_data.peer_id, rng)
        end
        if shooter_player_data.peer_id ~= ctx.my_id then
            if proj ~= nil then
                local lif = ComponentGetValue2(proj, "lifetime")
                if lif > 0 then
                    ComponentSetValue2(proj, "lifetime", lif * ctx.my_player.fps / shooter_player_data.fps)
                end
            end
            if life ~= nil then
                local lif = ComponentGetValue2(life, "lifetime")
                if lif > 0 then
                    ComponentSetValue2(life, "lifetime", lif * ctx.my_player.fps / shooter_player_data.fps)
                end
            end
            if body ~= nil then
                ComponentSetValue2(body, "destroy_body_if_entity_destroyed", true)
            end
        end
    end
    np.SetProjectileSpreadRNG(rng)
    if shooter_player_data.peer_id ~= ctx.my_id then
        for _, com in ipairs(EntityGetComponent(projectile_id, "LuaComponent") or {}) do
            local s = ComponentGetValue2(com, "script_source_file")
            if
                s == "data/entities/projectiles/deck/pebble_player.xml"
                or s == "data/scripts/animals/wand_ghost.lua"
                or s == "data/scripts/animals/pebble_player_spawn.lua"
            then
                EntityRemoveComponent(projectile_id, com)
            end
        end
    end
end

function OnProjectileFiredPost(
    shooter_id,
    projectile_id,
    rng,
    position_x,
    position_y,
    target_x,
    target_y,
    send_message,
    unknown1,
    multicast_index,
    unknown3
)
    if EntityHasTag(shooter_id, "ew_client") then
        local shooter_player_data = player_fns.get_player_data_by_local_entity_id(shooter_id)
        local vel = EntityGetFirstComponentIncludingDisabled(projectile_id, "VelocityComponent")
        if vel ~= nil then
            local x, y = ComponentGetValue2(vel, "mVelocity")
            local m = shooter_player_data.fps / ctx.my_player.fps
            ComponentSetValue2(vel, x * m, y * m)
        end
    end
end

util.add_cross_call("ew_is_wand_pickup", function()
    return ctx.is_wand_pickup
end)

util.add_cross_call("ew_pvp", function()
    return ctx.proxy_opt.pvp
end)

util.add_cross_call("ew_do_i_own", util.do_i_own)

local cross_force_send_inventory = false

util.add_cross_call("ew_api_force_send_inventory", function()
    cross_force_send_inventory = true
end)

function OnPausedChanged(paused, is_wand_pickup)
    ctx.is_paused = paused
    ctx.is_wand_pickup = is_wand_pickup

    local players = EntityGetWithTag("player_unit") or {}

    if players[1] then
        np.RegisterPlayerEntityId(players[1])
        --local inventory_gui = EntityGetFirstComponentIncludingDisabled(players[1], "InventoryGuiComponent")
        if paused then
            --EntitySetComponentIsEnabled(players[1], inventory_gui, false)
            np.EnableInventoryGuiUpdate(false)
            np.EnablePlayerItemPickUpper(false)
        else
            --EntitySetComponentIsEnabled(players[1], inventory_gui, true)
            np.EnableInventoryGuiUpdate(true)
            np.EnablePlayerItemPickUpper(true)
        end
    end
end

function OnWorldInitialized() -- This is called once the game world is initialized. Doesn't ensure any world chunks actually exist. Use OnPlayerSpawned to ensure the chunks around player have been loaded or created.
    if ctx.is_host then
        GameAddFlagRun("ew_flag_this_is_host")
    else
        GameRemoveFlagRun("ew_flag_this_is_host")
    end
    ctx.hook.on_world_initialized()
end

local last_chunk

local last_flex

function OnPlayerSpawned(player_entity) -- This runs when player entity has been created
    print("Initial player entity: " .. player_entity)
    if ctx.proxy_opt.home_on_players then
        EntityAddTag(player_entity, "homing_target")
    end

    if GlobalsGetValue("ew_player_count", "") == "" then
        GlobalsSetValue("ew_player_count", "1")
    end

    local x, y = EntityGetTransform(player_entity)
    ctx.initial_player_pos = { x = x, y = y }

    local my_player = player_fns.make_playerdata_for(player_entity, ctx.my_id)
    ctx.players[ctx.my_id] = my_player
    ctx.player_data_by_local_entity[player_entity] = my_player
    ctx.ready = true
    ctx.my_player = my_player

    EntityAddTag(player_entity, "ew_peer")

    if not GameHasFlagRun("ew_flag_notplayer_active") then
        EntityAddComponent2(
            player_entity,
            "LuaComponent",
            { script_wand_fired = "mods/quant.ew/files/resource/cbs/count_times_wand_fired.lua" }
        )
    end

    net.send_welcome()

    local item_pick = EntityGetFirstComponentIncludingDisabled(player_entity, "ItemPickUpperComponent")
    if item_pick ~= nil then
        ComponentSetValue2(item_pick, "is_immune_to_kicks", true)
    end

    ctx.hook.on_local_player_spawn(my_player)
    ctx.hook.on_should_send_updates()

    GamePrint("Noita Entangled Worlds version " .. version)

    OnPausedChanged(false, false)

    print("Game state entity: " .. GameGetWorldStateEntity())

    if not GameHasFlagRun("ew_flag_notplayer_active") then
        cos.player_cosmetics(player_entity)
        cos.player_color(player_entity)
    else
        local damage = ComponentGetValue2(player_entity, "DamageModelComponent")
        if damage ~= nil then
            ComponentSetValue2(damage, "ui_report_damage", false)
            ComponentSetValue2(damage, "hp", 2 ^ -38)
        end
        EntityInflictDamage(
            player_entity,
            1000000,
            "DAMAGE_CURSE",
            "dont rejoin",
            "NONE",
            0,
            0,
            GameGetWorldStateEntity()
        )
        GameAddFlagRun("ew_kill_player")
    end
    last_flex = ModSettingGet("quant.ew.flex")
    np.MagicNumbersSetValue("GRID_FLEXIBLE_MAX_UPDATES", last_flex)
    local controls_component = EntityGetFirstComponentIncludingDisabled(player_entity, "ControlsComponent")
    ComponentSetValue2(controls_component, "enabled", true)
    for _, child in ipairs(EntityGetAllChildren(player_entity) or {}) do
        local com = EntityGetFirstComponentIncludingDisabled(child, "LuaComponent")
        if com ~= nil and ComponentGetValue2(com, "script_source_file") == "data/scripts/perks/map.lua" then
            EntityRemoveComponent(child, com)
            EntityAddComponent2(child, "LuaComponent", { script_source_file = "data/scripts/perks/map.lua" })
            return
        end
    end
    inventory_helper.setup_inventory()
end

local last_n = 1

local function on_world_pre_update_inner()
    if ctx.my_player == nil or ctx.my_player.entity == nil then
        return
    end

    GlobalsSetValue("ew_player_rng", tostring(GameGetFrameNum()))

    net.update()
    if ctx.run_ended then
        return
    end

    local inventory_gui_comp = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "InventoryGuiComponent")
    if inventory_gui_comp and inventory_gui_comp ~= 0 then
        local inventory_open = ComponentGetValue2(inventory_gui_comp, "mActive")
        if ctx.is_inventory_open and not inventory_open then
            ctx.events.inventory_maybe_just_changed = true
        end
        ctx.is_inventory_open = inventory_open
    end

    if GameGetFrameNum() % 16 == 13 and not ctx.run_ended then
        if ctx.host_id == ctx.my_id then
            local new_chunk = tonumber(ModSettingGet("quant.ew.chunk_target") or 24) or 24
            if last_chunk ~= new_chunk then
                last_chunk = new_chunk
                np.MagicNumbersSetValue("STREAMING_CHUNK_TARGET", last_chunk)
            end
        end
        local new_flex = ModSettingGet("quant.ew.flex")
        if new_flex ~= last_flex then
            last_flex = new_flex
            np.MagicNumbersSetValue("GRID_FLEXIBLE_MAX_UPDATES", last_flex)
        end
        player_fns.respawn_if_necessary()
    end

    local sha_check = GameGetFrameNum() % 5 == 0 and inventory_helper.has_inventory_changed(ctx.my_player)
    if
        ctx.events.new_player_just_connected
        or ctx.events.inventory_maybe_just_changed
        or sha_check
        or cross_force_send_inventory
    then
        cross_force_send_inventory = false
        local inventory_state, spells = player_fns.serialize_items(ctx.my_player)
        if inventory_state ~= nil then
            net.send_player_inventory(inventory_state, spells)
        end
    end

    -- Perk sync
    if GameGetFrameNum() % 120 == 10 and not ctx.run_ended then
        local perk_data = perk_fns.get_my_perks()
        if perk_data ~= nil then
            net.send_player_perks(perk_data)
        end
    end

    if not ctx.run_ended then
        --local ti = GameGetRealWorldTimeSinceStarted()
        local n = EntitiesGetMaxID()
        for ent = last_n + 1, n do
            if EntityGetIsAlive(ent) then
                ctx.hook.on_new_entity(ent)
                local homing = EntityGetFirstComponentIncludingDisabled(ent, "HomingComponent")
                if homing ~= nil then
                    local projcom = EntityGetFirstComponentIncludingDisabled(ent, "ProjectileComponent")
                    if projcom ~= nil then
                        local whoshot = ComponentGetValue2(projcom, "mWhoShot")
                        if EntityHasTag(whoshot, "ew_notplayer") or GameHasFlagRun("ending_game_completed") then
                            ComponentSetValue2(homing, "target_tag", "ew_peer")
                        end
                    end
                end
            end
        end
        last_n = n
        --local tf = GameGetRealWorldTimeSinceStarted()
        if ctx.is_host then
            ctx.hook.on_world_update_host()
        else
            ctx.hook.on_world_update_client()
        end
        --local tf2 = GameGetRealWorldTimeSinceStarted()
        ctx.hook.on_world_update()
        --[[local tf3 = GameGetRealWorldTimeSinceStarted()
        if GameGetFrameNum() % 5 == 0 then
            GamePrint(
                math.ceil((tf - ti) * 1000000)
                    .. " "
                    .. math.ceil((tf2 - tf) * 1000000)
                    .. " "
                    .. math.ceil((tf3 - tf2) * 1000000)
            )
        end]]
    end

    perk_fns.on_world_update()
    wake_up_waiting_threads(1)
end

local entity_sync

function OnWorldPreUpdate() -- This is called every time the game is about to start updating the world
    if net.connect_failed then
        if GameGetFrameNum() % 180 == 0 then
            GamePrint("Entangled Worlds mod is enabled, but it couldn't connect to proxy!")
            GamePrint("You need to start the proxy and join the lobby first.")
            GamePrint("If you want to play singleplayer, disable the mod and start a new game.")
        end
        return
    end
    util.tpcall(on_world_pre_update_inner)
    local n = math.floor(tonumber(ModSettingGet("quant.ew.entity_sync") or 2) or 2 + 0.5)
    if entity_sync ~= n then
        entity_sync = n
        ewext.send_sync_rate(entity_sync)
    end
    --[[if InputIsKeyJustDown(11) then
        local x, y = EntityGetTransform(ctx.my_player.entity)
        for _ = 1, 16 do
            EntityLoad("data/entities/animals/longleg.xml", x, y)
        end
    end]]
end

local function on_world_post_update_inner()
    if ctx.my_player == nil or ctx.my_player.entity == nil then
        return
    end

    if ctx.run_ended then
        return
    end

    ctx.hook.on_world_update_post()

    local times_wand_fired = tonumber(GlobalsGetValue("ew_wand_fired", "0"))
    GlobalsSetValue("ew_wand_fired", "0")
    local wand = player_fns.get_active_held_item(ctx.my_player.entity)
    local ability = EntityGetFirstComponentIncludingDisabled(wand, "AbilityComponent")
    if
        times_wand_fired > 0
        or (
            wand ~= nil
            and EntityHasTag(wand, "card_action")
            and ability ~= nil
            and ComponentGetValue2(ability, "mCastDelayStartFrame") == GameGetFrameNum()
        )
    then
        fire()
    end
    if ability ~= nil then
        last_mana = ComponentGetValue2(ability, "mana")
    else
        last_mana = -1
    end
end

function OnWorldPostUpdate() -- This is called every time the game has finished updating the world
    if net.connect_failed then
        return
    end
    util.tpcall(on_world_post_update_inner)
    ctx.events = {}
    net.proxy_send("flush", "")
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

    if not net.connect_failed then
        util.tpcall(load_modules)
        print("Entangled worlds init complete")
    end
end

function OnModInit()
    load_extra_modules()
end

function OnModPostInit()
    ctx.hook.on_late_init()
end

function OnPlayerDied(player_entity)
    ctx.hook.on_player_died(player_entity)
    print("player died")
end
