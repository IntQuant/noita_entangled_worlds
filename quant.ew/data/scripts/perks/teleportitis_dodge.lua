dofile_once("data/scripts/lib/utilities.lua")

local sensor_range = 20
local teleport_range = 90
local time_active = 1
local time_cooldown = 130

local entity_id = GetUpdatedEntityID()
local root_id = EntityGetRootEntity(entity_id)
local pos_x, pos_y = EntityGetTransform( entity_id )

local function teleport(from_x, from_y, to_x, to_y)
    EntitySetTransform(root_id, to_x, to_y)
    EntityLoad("data/entities/particles/teleportation_source.xml", from_x, from_y)
    EntityLoad("data/entities/particles/teleportation_target.xml", to_x, to_y)
    GamePlaySound("data/audio/Desktop/misc.bank","misc/teleport_use", to_x, to_y)

    -- punch a hole to make sure player doesn't get stuck
    LoadPixelScene("data/biome_impl/teleportitis_dodge_hole.png", "", to_x-3, to_y-12, "", true)
end

-- toggles vfx and sets script exec time
local function set_cooldown(on_cooldown, frames)
    EntitySetComponentsWithTagEnabled(entity_id, "teleportitis_dodge_vfx", not on_cooldown)
    component_write( GetUpdatedComponentID(), {
        execute_every_n_frame = frames,
        mNextExecutionTime = GameGetFrameNum() + frames,
    })
end

local genome = EntityGetFirstComponentIncludingDisabled(entity_id, "GenomeDataComponent")
local my_herd
if genome ~= nil then
    my_herd = ComponentGetValue(genome, "herd_id")
end


-- look for enemy projectiles
for _,proj_id in ipairs(EntityGetInRadiusWithTag( pos_x, pos_y, sensor_range, "projectile" )) do
    local comp_proj = EntityGetFirstComponent(proj_id, "ProjectileComponent")
    if comp_proj ~= nil then
        local shooter_id = tonumber(ComponentGetValue(comp_proj, "mWhoShot"))
        local herd_id = tonumber(ComponentGetValue(comp_proj, "mShooterHerdId") or -1)
        if shooter_id ~= 0 and shooter_id ~= root_id and herd_id ~= my_herd then
            -- found. let's prep for teleport
            local x = 0
            local y = 0

            -- teleport direction from player aim
            local aim_comp = EntityGetFirstComponent(root_id, "SpriteComponent", "aiming_reticle")
            component_read( aim_comp, { offset_x = 0, offset_y = 0 }, function(comp)
                x = comp.offset_x
                y = comp.offset_y - 4
                x, y = vec_normalize(x, y)
                x, y = vec_mult(x, y, teleport_range)
                x, y = vec_rotate(x, y, -math.pi * 0.5 * sign(x)) -- rotate 90 towards top
                local did_hit
                did_hit,x,y = RaytracePlatforms(pos_x, pos_y, pos_x + x, pos_y + y)
            end)

            -- reel it back a bit so we're less likely to end up inside a wall
            local back_x = pos_x - x
            local back_y = pos_y - y
            -- go back 20px from teleport target but don't go beyond initial teleport position
            local back_distance = math.min(20, get_magnitude(back_x, back_y))
            back_x, back_y = vec_normalize(back_x, back_y)
            x = x + back_x * back_distance
            y = y + back_y * back_distance

            -- teleport
            teleport(pos_x, pos_y, x, y)
            set_cooldown(true, time_cooldown)
            return
        end
    end
end

-- cooldown off
set_cooldown(false, time_active)