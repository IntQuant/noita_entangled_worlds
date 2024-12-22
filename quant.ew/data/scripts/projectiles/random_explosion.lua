dofile_once("data/scripts/lib/utilities.lua")

local entity_id    = GetUpdatedEntityID()

local projectilecomponents = EntityGetComponent( entity_id, "ProjectileComponent" )

local opts = {"acidshot","alcohol_blast","black_hole_big","cloud_thunder","cloud_acid","cloud_blood","cloud_water","death_cross","death_cross_big","fireball","firebomb","grenade","grenade_tier_2","grenade_tier_3","lightning","meteor","nuke","regeneration_field","rocket","rocket_tier_2","rocket_tier_3","tentacle_portal","thunder_blast","wall_square","xray"}
local rndv = 0
for _, var in ipairs(EntityGetComponent(entity_id, "VariableStorageComponent") or {}) do
    if ComponentGetValue(var, "name") == "ew_transmutation" then
        rndv = ComponentGetValue(var, "value_int")
    end
end
SetRandomSeed( rndv + 2533, rndv - 36 )
local rnd = Random( 1, #opts )

local result = "data/entities/projectiles/deck/" .. opts[rnd] .. ".xml"

if ( projectilecomponents ~= nil ) then
    for i,comp_id in ipairs( projectilecomponents ) do
        ComponentSetValue2( comp_id, "on_collision_spawn_entity", true )
        ComponentSetValue2( comp_id, "spawn_entity_is_projectile", true )

        ComponentSetValue2( comp_id, "spawn_entity", result )
    end
end