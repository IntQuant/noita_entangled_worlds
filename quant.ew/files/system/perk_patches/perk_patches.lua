local rpc = net.new_rpc_namespace()

local module = {}

ModLuaFileAppend("data/scripts/perks/perk_list.lua", "mods/quant.ew/files/system/perk_patches/append/perks_common.lua")
ModLuaFileAppend("data/scripts/perks/perk_utilities.lua", "mods/quant.ew/files/system/perk_patches/append/cosmetics_append.lua")

if ctx.proxy_opt.game_mode == "shared_health" then
    print("Loading extra perk patches for shared health mode")
    ModLuaFileAppend("data/scripts/perks/perk_list.lua", "mods/quant.ew/files/system/perk_patches/append/perks_shared.lua")
    --else
    --print("Loading extra perk patches for local health mode")
    --ModLuaFileAppend("data/scripts/perks/perk_list.lua", "mods/quant.ew/files/system/perk_patches/append/perks_local.lua")
end

if not ctx.is_host or not ctx.proxy_opt.randomize_perks then
    --print("Hiding telekinesis")
    ModLuaFileAppend("data/scripts/perks/perk_list.lua", "mods/quant.ew/files/system/perk_patches/append/perks_no_telekinesis.lua")
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

util.add_cross_call("ew_perks_modify_max_hp", rpc.modify_max_hp)

util.add_cross_call("ew_ff", function()
    return ctx.proxy_opt.friendly_fire
end)

util.add_cross_call("ew_perk_ban_list", function()
    return ctx.proxy_opt.perk_ban_list
end)

util.add_cross_call("ew_randomize_perks", function()
    return ctx.proxy_opt.randomize_perks
end)

local function become_rat(entity_who_picked)
    local child_id = EntityLoad( "data/entities/verlet_chains/tail/verlet_tail.xml", x, y )
    EntityAddTag( child_id, "perk_entity" )
    EntityAddChild( entity_who_picked, child_id )

    local platformingcomponents = EntityGetComponent( entity_who_picked, "CharacterPlatformingComponent" )
    if( platformingcomponents ~= nil ) then
        for i,component in ipairs(platformingcomponents) do
            local run_speed = tonumber( ComponentGetMetaCustom( component, "run_velocity" ) ) * 1.15
            local vel_x = math.abs( tonumber( ComponentGetMetaCustom( component, "velocity_max_x" ) ) ) * 1.15

            local vel_x_min = 0 - vel_x
            local vel_x_max = vel_x

            ComponentSetMetaCustom( component, "run_velocity", run_speed )
            ComponentSetMetaCustom( component, "velocity_min_x", vel_x_min )
            ComponentSetMetaCustom( component, "velocity_max_x", vel_x_max )
        end
    end
end

local function become_fungus(entity_who_picked)
    EntitySetComponentsWithTagEnabled( entity_who_picked, "player_hat", true )
    EntitySetComponentsWithTagEnabled( entity_who_picked, "player_hat2_shadow", false )

    local damagemodels = EntityGetComponent( entity_who_picked, "DamageModelComponent" )
    if( damagemodels ~= nil ) then
        for i,damagemodel in ipairs(damagemodels) do
            local explosion_resistance = tonumber(ComponentObjectGetValue( damagemodel, "damage_multipliers", "explosion" ))
            explosion_resistance = explosion_resistance * 0.9
            ComponentObjectSetValue( damagemodel, "damage_multipliers", "explosion", tostring(explosion_resistance) )
        end
    end
end

local function become_luuki(entity_who_picked)
    EntitySetComponentsWithTagEnabled( entity_who_picked, "lukki_enable", true )
    local comp = EntityGetFirstComponent( entity_who_picked, "SpriteComponent", "lukki_disable" )
    if ( comp ~= nil ) then
        ComponentSetValue2( comp, "alpha", 0.0 )
    end
    local platformingcomponents = EntityGetComponent( entity_who_picked, "CharacterPlatformingComponent" )
    if( platformingcomponents ~= nil ) then
        for i,component in ipairs(platformingcomponents) do
            local run_speed = tonumber( ComponentGetMetaCustom( component, "run_velocity" ) ) * 1.1
            local vel_x = math.abs( tonumber( ComponentGetMetaCustom( component, "velocity_max_x" ) ) ) * 1.1

            local vel_x_min = 0 - vel_x
            local vel_x_max = vel_x

            ComponentSetMetaCustom( component, "run_velocity", run_speed )
            ComponentSetMetaCustom( component, "velocity_min_x", vel_x_min )
            ComponentSetMetaCustom( component, "velocity_max_x", vel_x_max )
        end
    end
end

local function become_ghost(entity_who_picked)
    local child_id = EntityLoad( "data/entities/misc/perks/ghostly_ghost.xml", x, y )
    local child_id2 = EntityLoad( "data/entities/misc/perks/tiny_ghost_extra.xml", x, y )
    EntityAddTag( child_id, "perk_entity" )
    EntityAddTag( child_id2, "perk_entity" )
    EntityAddChild( entity_who_picked, child_id )
    EntityAddChild( entity_who_picked, child_id2 )

    local platformingcomponents = EntityGetComponent( entity_who_picked, "CharacterDataComponent" )
    if( platformingcomponents ~= nil ) then
        for i,component in ipairs(platformingcomponents) do
            local fly_time = ComponentGetValue2( component, "fly_recharge_spd" ) * 1.15
            ComponentSetValue2( component, "fly_recharge_spd", fly_time )
        end
    end
end

local function lose_halo(entity_who_picked)
    local damagemodels = EntityGetComponent( entity_who_picked, "DamageModelComponent" )
    -- clear decoration
    for _,v in ipairs(EntityGetAllChildren(entity_who_picked)) do
        if EntityGetName(v) == "halo" then
            EntityRemoveFromParent(v)
            EntityKill(v)
            break
        end
    end

    -- remove fire resistance
    if( damagemodels ~= nil ) then
        for i,damagemodel in ipairs(damagemodels) do
            local fire_resistance = tonumber(ComponentObjectGetValue( damagemodel, "damage_multipliers", "fire" ))
            fire_resistance = fire_resistance / 0.9
            --print(fire_resistance)
            ComponentObjectSetValue( damagemodel, "damage_multipliers", "fire", tostring(fire_resistance) )

            local holy_resistance = tonumber(ComponentObjectGetValue( damagemodel, "damage_multipliers", "holy" ))
            holy_resistance = holy_resistance / 0.9
            ComponentObjectSetValue( damagemodel, "damage_multipliers", "holy", tostring(holy_resistance) )
        end
    end
end

local function gain_halo(entity_who_picked, angle)
    local damagemodels = EntityGetComponent( entity_who_picked, "DamageModelComponent" )
    -- spawn decoration
    local child_id
    if angle then
        child_id = EntityLoad( "data/entities/misc/perks/player_halo_light.xml", x, y )
    else
        child_id = EntityLoad( "data/entities/misc/perks/player_halo_dark.xml", x, y )
    end
    if child_id ~= nil then
        EntityAddChild( entity_who_picked, child_id )
    end

    -- add fire & holy resistance
    if( damagemodels ~= nil ) then
        for i,damagemodel in ipairs(damagemodels) do
            local fire_resistance = tonumber(ComponentObjectGetValue( damagemodel, "damage_multipliers", "fire" ))
            fire_resistance = fire_resistance * 0.9
            --print(fire_resistance)
            ComponentObjectSetValue( damagemodel, "damage_multipliers", "fire", tostring(fire_resistance) )

            local holy_resistance = tonumber(ComponentObjectGetValue( damagemodel, "damage_multipliers", "holy" ))
            holy_resistance = holy_resistance * 0.9
            ComponentObjectSetValue( damagemodel, "damage_multipliers", "holy", tostring(holy_resistance) )
        end
    end
end

function rpc.send_mutations(ghost, luuki, rat, fungus, halo)
    local last = ctx.players[ctx.rpc_peer_id].mutations
    ctx.players[ctx.rpc_peer_id].mutations = {ghost = ghost, luuki = luuki, rat = rat, fungus = fungus, halo = halo}
    local ent = ctx.rpc_player_data.entity
    if ghost and not last.ghost then
        become_ghost(ent)
    end
    if luuki and not last.luuki then
        become_luuki(ent)
    end
    if rat and not last.rat then
        become_rat(ent)
    end
    if fungus and not last.fungus then
        become_fungus(ent)
    end
    if math.abs(halo) < 3 and math.abs(last.halo) >= 3 then
        lose_halo(ent)
    elseif math.abs(halo) >= 3 and math.abs(last.halo) < 3 then
        gain_halo(ent, halo >= 3)
    end
end

function module.on_world_update()
    if GameGetFrameNum() % 60 == 26 then
        rpc.send_mutations(
                tonumber(GlobalsGetValue("PLAYER_GHOSTNESS_LEVEL", "0")) >= 3,
                tonumber(GlobalsGetValue("PLAYER_LUKKINESS_LEVEL", "0")) >= 3,
                tonumber(GlobalsGetValue("PLAYER_RATTINESS_LEVEL", "0")) >= 3,
                tonumber(GlobalsGetValue("PLAYER_FUNGAL_LEVEL", "0")) >= 3,
                tonumber(GlobalsGetValue("PLAYER_HALO_LEVEL", "0"))
        )
    end
end

function module.on_client_spawned(peer_id, player_data)
    local ent = player_data.entity
    if player_data.mutations.ghost then
        become_ghost(ent)
    end
    if player_data.mutations.luuki then
        become_luuki(ent)
    end
    if player_data.mutations.rat then
        become_rat(ent)
    end
    if player_data.mutations.fungus then
        become_fungus(ent)
    end
    if math.abs(player_data.mutations.halo) == 3 then
        gain_halo(ent, player_data.mutations.halo == 3)
    end
end

return module