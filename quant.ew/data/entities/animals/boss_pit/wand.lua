dofile_once("data/scripts/lib/utilities.lua")

local entity_id    = GetUpdatedEntityID()
local x, y, a = EntityGetTransform( entity_id )

local p = EntityGetWithTag( "ew_peer" )
local closest
local vs
for _, player in ipairs(p) do
    if not EntityHasTag(player, "ew_notplayer") then
        local px, py = EntityGetTransform(player)
        local r = px * px + py * py
        if closest == nil or r < closest then
            closest = r
            vs = player
        end
    end
end
local comps = EntityGetComponent( entity_id, "VariableStorageComponent" )
local proj = ""
local mult = 1.0
if ( comps ~= nil ) then
    for i,v in ipairs( comps ) do
        local n = ComponentGetValue2( v, "name" )
        if ( n == "memory" ) then
            proj = ComponentGetValue2( v, "value_string" )
        elseif ( n == "mult" ) then
            mult = ComponentGetValue2( v, "value_float" )
        end
    end
end

local length = 300
local homing = EntityGetComponent( entity_id, "HomingComponent" )
if ( homing == nil ) then
    length = 500
end

if ( vs ~= nil ) and ( #proj > 0 ) then
    local px,py = EntityGetTransform( vs )
    local dir = 0 - math.atan2( py - y, px - x )

    local vel_x = math.cos( dir ) * length * mult
    local vel_y = 0 - math.sin( dir ) * length * mult
    local pid = shoot_projectile_from_projectile( entity_id, proj, x, y, vel_x, vel_y )

    edit_component( pid, "ProjectileComponent", function(comp,vars)
        local d = ComponentGetValue2( comp, "damage" )
        d = d + 0.2
        ComponentSetValue2( comp, "damage", d )
    end)
end