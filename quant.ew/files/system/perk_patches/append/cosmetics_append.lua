function add_rattiness_level(entity_who_picked)
    local peer_id = EntityGetName(entity_who_picked)
    if peer_id ~= "DEBUG_NAME:player" then
        return
    end
    local x,y = EntityGetTransform( entity_who_picked )
    local rattiness = tonumber( GlobalsGetValue( "PLAYER_RATTINESS_LEVEL", "0" ) )
    rattiness = rattiness + 1
    GlobalsSetValue( "PLAYER_RATTINESS_LEVEL", tostring( rattiness ) )

    if ( rattiness == 3 ) then
        local child_id = EntityLoad( "data/entities/verlet_chains/tail/verlet_tail.xml", x, y )
        EntityAddTag( child_id, "perk_entity" )
        EntityAddChild( entity_who_picked, child_id )
        AddFlagPersistent( "player_status_ratty" )

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
end

function add_funginess_level(entity_who_picked)
    local peer_id = EntityGetName(entity_who_picked)
    if peer_id ~= "DEBUG_NAME:player" then
        return
    end
    local funginess = tonumber( GlobalsGetValue( "PLAYER_FUNGAL_LEVEL", "0" ) )
    funginess = funginess + 1
    GlobalsSetValue( "PLAYER_FUNGAL_LEVEL", tostring( funginess ) )

    if ( funginess == 3 ) then
        EntitySetComponentsWithTagEnabled( entity_who_picked, "player_hat", true )
        EntitySetComponentsWithTagEnabled( entity_who_picked, "player_hat2_shadow", false )

        AddFlagPersistent( "player_status_funky" )

        local damagemodels = EntityGetComponent( entity_who_picked, "DamageModelComponent" )
        if( damagemodels ~= nil ) then
            for i,damagemodel in ipairs(damagemodels) do
                local explosion_resistance = tonumber(ComponentObjectGetValue( damagemodel, "damage_multipliers", "explosion" ))
                explosion_resistance = explosion_resistance * 0.9
                ComponentObjectSetValue( damagemodel, "damage_multipliers", "explosion", tostring(explosion_resistance) )
            end
        end
    end
end

function add_ghostness_level(entity_who_picked)
    local peer_id = EntityGetName(entity_who_picked)
    if peer_id ~= "DEBUG_NAME:player" then
        return
    end
    local x,y = EntityGetTransform( entity_who_picked )
    local ghostness = tonumber( GlobalsGetValue( "PLAYER_GHOSTNESS_LEVEL", "0" ) )
    ghostness = ghostness + 1
    GlobalsSetValue( "PLAYER_GHOSTNESS_LEVEL", tostring( ghostness ) )

    if ( ghostness == 3 ) then
        local child_id = EntityLoad( "data/entities/misc/perks/ghostly_ghost.xml", x, y )
        local child_id2 = EntityLoad( "data/entities/misc/perks/tiny_ghost_extra.xml", x, y )
        EntityAddTag( child_id, "perk_entity" )
        EntityAddTag( child_id2, "perk_entity" )
        EntityAddChild( entity_who_picked, child_id )
        EntityAddChild( entity_who_picked, child_id2 )

        AddFlagPersistent( "player_status_ghostly" )

        local platformingcomponents = EntityGetComponent( entity_who_picked, "CharacterDataComponent" )
        if( platformingcomponents ~= nil ) then
            for i,component in ipairs(platformingcomponents) do
                local fly_time = ComponentGetValue2( component, "fly_recharge_spd" ) * 1.15
                ComponentSetValue2( component, "fly_recharge_spd", fly_time )
            end
        end
    end
end

function add_lukkiness_level(entity_who_picked)
    local peer_id = EntityGetName(entity_who_picked)
    if peer_id ~= "DEBUG_NAME:player" then
        return
    end
    local x,y = EntityGetTransform( entity_who_picked )
    local lochness = tonumber( GlobalsGetValue( "PLAYER_LUKKINESS_LEVEL", "0" ) )
    lochness = lochness + 1
    GlobalsSetValue( "PLAYER_LUKKINESS_LEVEL", tostring( lochness ) )

    if ( lochness == 3 ) then
        EntitySetComponentsWithTagEnabled( entity_who_picked, "lukki_enable", true )
        AddFlagPersistent( "player_status_lukky" )

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
end


function add_halo_level(entity_who_picked, amount, set_to_this_value)
    -- NOTE: Player may be able to play the system slightly by toggling
    -- the resistance boost on/off in combination with other boosts.
    -- It's most likely not worth the effort though...

    local peer_id = EntityGetName(entity_who_picked)
    if peer_id ~= "DEBUG_NAME:player" then
        return
    end

    local x,y = EntityGetTransform( entity_who_picked )
    local damagemodels = EntityGetComponent( entity_who_picked, "DamageModelComponent" )

    local lvl = tonumber( GlobalsGetValue( "PLAYER_HALO_LEVEL", "0" ) )
    local halo_gained =    (lvl == 2 and amount == 1) or (lvl == -2 and amount == -1)
    local halo_lost = (lvl == -3 and amount == 1) or (lvl == 3 and amount == -1)

    lvl = lvl + amount

    if ( set_to_this_value ~= nil ) then
        lvl = set_to_this_value

        if ( lvl == 0 ) then
            halo_lost = true
            halo_gained = false
        end
    end

    GlobalsSetValue( "PLAYER_HALO_LEVEL", tostring( lvl ) )

    if halo_lost then
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
    elseif halo_gained then
        -- spawn decoration
        local child_id
        if lvl >= 3 then
            child_id = EntityLoad( "data/entities/misc/perks/player_halo_light.xml", x, y )
        elseif lvl <= -3 then
            child_id = EntityLoad( "data/entities/misc/perks/player_halo_dark.xml", x, y )
        end
        if child_id ~= nil then
            EntityAddChild( entity_who_picked, child_id )
            AddFlagPersistent( "player_status_halo" )
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
end