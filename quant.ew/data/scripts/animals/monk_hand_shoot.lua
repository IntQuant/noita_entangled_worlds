dofile_once("data/scripts/lib/utilities.lua")

local range = 200
local projectile_velocity = 20
local scatter = 0.2

local entity_id = GetUpdatedEntityID()
local root_id = EntityGetRootEntity(entity_id)
local pos_x, pos_y = EntityGetFirstHitboxCenter( entity_id )

-- locate nearest enemy
local enemy, enemy_x, enemy_y
local min_dist = 9999
for _,id in pairs(EntityGetInRadiusWithTag(pos_x, pos_y, range, "mortal")) do
    -- is target a valid enemy
    if EntityGetComponent(id, "GenomeDataComponent") ~= nil and EntityGetComponent(root_id, "GenomeDataComponent") ~= nil and EntityGetHerdRelation(root_id, id) < 40 then
        local x, y = EntityGetFirstHitboxCenter( id )
        local dist = get_distance(pos_x, pos_y, x, y)
        if dist < min_dist then
            min_dist = dist
            enemy = id
            enemy_x = x
            enemy_y = y
        end
    end
end

-- check los
local can_shoot = false
if enemy then can_shoot = not RaytraceSurfacesAndLiquiform(pos_x, pos_y, enemy_x, enemy_y) end

-- hand/shooting state & animation control
edit_component2( entity_id, "SpriteComponent", function(comp,vars)
    -- if enemy is not visible then open hand
    local hand_check = ComponentGetValue2( comp, "rect_animation" )
    local hand_open = false

    if ( hand_check ~= nil ) and ( hand_check == "open" ) then
        hand_open = true
    end

    if not can_shoot then
        if not hand_open then
            ComponentSetValue2( comp, "rect_animation", "open")
            --EntitySetComponentsWithTagEnabled( entity_id, "enabled_when_attacking", false )
        end
        return
    end

    -- prepare to shoot
    if hand_open then
        ComponentSetValue2( comp, "rect_animation", "close")
        --EntitySetComponentsWithTagEnabled( entity_id, "enabled_when_attacking", true )
        can_shoot = false
        return
    end
end)

if not can_shoot then return end

-- direction
local vel_x, vel_y = vec_sub(enemy_x, enemy_y, pos_x, pos_y)
vel_x,vel_y = vec_normalize(vel_x, vel_y)

-- offset to shoot direction to avoid colliding with hand
pos_x = pos_x + vel_x * 8
pos_y = pos_y + vel_y * 8

-- scatter
SetRandomSeed(pos_x + GameGetFrameNum(), pos_y)
vel_x,vel_y = vec_rotate(vel_x,vel_y, rand(-scatter, scatter))

-- apply velocity & shoot
vel_x,vel_y = vec_mult(vel_x,vel_y, projectile_velocity)
shoot_projectile( root_id, "data/entities/projectiles/orb_green_accelerating.xml", pos_x, pos_y, vel_x, vel_y)
--EntityLoad("data/entities/particles/particle_explosion/main_swirly_green.xml", pos_x, pos_y)

-- delay randomly so that multiple hands shoot at different times
ComponentSetValue(GetUpdatedComponentID(), "execute_every_n_frame", 55 + Random(10))