dofile_once("data/scripts/lib/utilities.lua")

local entity_id    = GetUpdatedEntityID()
local x, y = EntityGetTransform( GetUpdatedEntityID() )

local dm = 1.0

edit_component( entity_id, "HitboxComponent", function(comp,vars)
    dm = ComponentGetValue2( comp, "damage_multiplier" )

    if ( dm < 1.0 ) then
        dm = math.min( 1.0, dm + 0.35 )
    end

    ComponentSetValue2( comp, "damage_multiplier", dm )
end)

EntitySetComponentsWithTagEnabled( entity_id, "invincible", false )

local state = 0
local p = ""
local pathfinding_frames_stuck = 0
local comps = EntityGetComponent( entity_id, "VariableStorageComponent" )
if ( comps ~= nil ) then
    for i,v in ipairs( comps ) do
        local n = ComponentGetValue2( v, "name" )
        if ( n == "state" ) then
            state = ComponentGetValue2( v, "value_int" )

            state = (state + 1) % 10

            ComponentSetValue2( v, "value_int", state )
        elseif ( n == "memory" ) then
            --print( ComponentGetValue2( v, "value_string" ) )
            p = ComponentGetValue2( v, "value_string" )

            if ( #p == 0 ) then
                p = "data/entities/projectiles/enlightened_laser_darkbeam.xml"
                ComponentSetValue2( v, "value_string", p )
            end
        elseif ( n == "pathfinding_frames_stuck" ) then
            pathfinding_frames_stuck = ComponentGetValue2( v, "value_int" )
        end
    end
end

SetRandomSeed( x, y * GameGetFrameNum() )

if ( state == 1 ) then
    if ( #p > 0 ) then
        local angle = Random( 1, 200 ) * math.pi
        local vel_x = math.cos( angle ) * 100
        local vel_y = 0 - math.cos( angle ) * 100

        --[[
            "data/entities/projectiles/deck/rocket.xml"
            "data/entities/projectiles/deck/rocket_tier_2.xml"
            "data/entities/projectiles/deck/rocket_tier_3.xml"
            "data/entities/projectiles/deck/grenade.xml"
            "data/entities/projectiles/deck/grenade_tier_2.xml"
            "data/entities/projectiles/deck/grenade_tier_3.xml"
            "data/entities/projectiles/deck/rubber_ball.xml"
        ]]--
        local spells = { "rocket", "rocket_tier_2", "rocket_tier_3", "grenade", "grenade_tier_2", "grenade_tier_3", "rubber_ball" }
        local rnd = Random( 1, #spells )
        local path = "data/entities/projectiles/deck/" .. spells[rnd] .. ".xml"

        local wid = shoot_projectile( entity_id, "data/entities/animals/boss_pit/wand.xml", x, y, vel_x, vel_y )
        edit_component( wid, "VariableStorageComponent", function(comp,vars)
            ComponentSetValue2( comp, "value_string", path )
        end)

        EntityAddComponent( wid, "HomingComponent",
        {
            homing_targeting_coeff = "30.0",
            homing_velocity_multiplier = "0.16",
            target_tag = "ew_peer",
        } )

        if ( string.find( path, "rocket" ) ~= nil ) then
            EntityAddComponent( wid, "VariableStorageComponent",
            {
                name = "mult",
                value_float = 0.5,
            } )
        else
            EntityAddComponent( wid, "VariableStorageComponent",
            {
                name = "mult",
                value_float = 1.2,
            } )
        end
    end
elseif ( state == 7 ) then
    if ( Random( 1, 10 ) == 5 ) or ( 1 == 1 ) then

        --[[
        "data/entities/projectiles/orb_poly.xml"
        "data/entities/projectiles/orb_neutral.xml"
        "data/entities/projectiles/orb_tele.xml"
        "data/entities/projectiles/orb_dark.xml"
        ]]--

        -- if we're stuck shoot blackholes towards player
        if( pathfinding_frames_stuck > 160 ) then

            -- we're stuck, lets hunt for that connoisseur of cheese
            local path = "data/entities/projectiles/remove_ground.xml"
            shoot_projectile( entity_id, path, x, y, 0, 0 )

        else

            -- standard logic
            local spells = { "orb_poly", "orb_neutral", "orb_tele", "orb_dark" }
            local rnd = Random( 1, #spells )
            local path = "data/entities/projectiles/" .. spells[rnd] .. ".xml"

            local arc = math.pi * 0.25
            local offset = math.pi * ( Random( 1, 10 ) * 0.1 )

            for i=0,7 do
                local vel_x = math.cos( arc * i + offset ) * 300
                local vel_y = 0 - math.sin( arc * i + offset ) * 300
                shoot_projectile( entity_id, path, x, y, vel_x, vel_y )
            end
        end
    end
end