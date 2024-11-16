dofile_once("data/scripts/lib/utilities.lua")

local entity_id    = GetUpdatedEntityID()
local pos_x, pos_y = EntityGetTransform( entity_id )

EntitySetComponentsWithTagEnabled( entity_id, "transmutation", true )

local convertcomponents = EntityGetComponent( entity_id, "MagicConvertMaterialComponent" )

local rnd = pos_x + pos_y
for _, var in ipairs(EntityGetComponent(entity_id, "VariableStorageComponent") or {}) do
    if ComponentGetValue(var, "name") == "ew_transmutation" then
        rnd = ComponentGetValue(var, "value_int")
    end
end
SetRandomSeed( rnd + 436, rnd - 3252 )
local material_options = { "water", "oil", "lava", "acid", "radioactive_liquid", "slime", "sand", "alcohol", "blood", "snow", "blood_worm", "blood_fungi", "burning_powder", "honey", "fungi", "diamond", "brass", "silver" }
local material_options_rare = { "acid", "magic_liquid_teleportation", "magic_liquid_polymorph", "magic_liquid_random_polymorph", "magic_liquid_berserk", "magic_liquid_charm", "magic_liquid_invisibility" }
local rare = false

rnd = Random( 1, 100 )

if ( rnd > 98 ) then
    rare = true
end

local material_string = "water"

if (rare == false) then
    rnd = Random( 1, #material_options )
    material = material_options[rnd]
else
    rnd = Random( 1, #material_options_rare )
    material = material_options_rare[rnd]
end

material = CellFactory_GetType( material )

if ( convertcomponents ~= nil ) then
    for key,comp_id in pairs(convertcomponents) do
        local mat_name = tonumber( ComponentGetValue( comp_id, "from_material" ) )
        --local smoke_id = CellFactory_GetType( "smoke" )

        if (material == mat_name) then
            --ComponentSetValue( comp_id, "to_material", smoke_id )
        else
            ComponentSetValue( comp_id, "to_material", material )
        end
    end
end

edit_component( entity_id, "LuaComponent", function(comp,vars)
    EntitySetComponentIsEnabled( entity_id, comp, false )
end)