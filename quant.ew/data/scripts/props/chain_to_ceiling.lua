--[[
Creates a chain to ceiling from a given offset.
Script looks for variable storages named "chain_n_x" and "chain_n_y" where n is 0...9 and generates one or more chains based on them.
If no variable storages are found a single chain with 0 offsets is created.
NOTE: Requires PhysicsBody2Component with body_id 100

XML:
<LuaComponent
    script_source_file="data/scripts/props/chain_to_ceiling.lua"
    execute_on_added="1"
    execute_every_n_frame="5"
    execute_times="-1"
    >
</LuaComponent>

<VariableStorageComponent
    name="chain_1_x"
    value_int="-8"
    >
</VariableStorageComponent>
<VariableStorageComponent
    name="chain_1_y"
    value_int="4"
    >
</VariableStorageComponent>

--]]

dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local pos_x, pos_y = EntityGetTransform( entity_id )

local segment_length = 16
local search_dist = 200
local break_force_mid = 1.25
local break_force_end = 3
local break_distance_end = 20
local break_distance_mid = 3

if not DoesWorldExistAt( pos_x, pos_y - search_dist, pos_x, pos_y ) then return end

local count = 0
local function get_random_chain()
    count = count + 1
    local n = ProceduralRandomi(pos_x + entity_id, pos_y + count, 0, 2)
    return "data/props_breakable_gfx/chain_vertical_16_0" .. n .. ".png"
end

-- get offsets from components (if available)
local offsets_x = { }
local offsets_y = { }
for i=0,9 do
    -- get stored values based on naming convention
    local s = "chain_" .. i
    local comp_x = get_variable_storage_component(entity_id, s .. "_x")
    local comp_y = get_variable_storage_component(entity_id, s .. "_y")
    if comp_x ~= nil then offsets_x[#offsets_x+1] = ComponentGetValue2(comp_x, "value_int") end
    if comp_y ~= nil then offsets_y[#offsets_y+1] = ComponentGetValue2(comp_y, "value_int") end
end
offsets_x[1] = offsets_x[1] or 0
offsets_y[1] = offsets_y[1] or 0

-- modified joint strength (optional)
local comp_strength = get_variable_storage_component(entity_id, "chain_strength_multiplier")
if comp_strength ~= nil then
    local mult = ComponentGetValue2(comp_strength, "value_float")
    break_force_mid = break_force_mid * mult
    break_force_end = break_force_end * mult
end


local mat = CellFactory_GetType( "metal_chain_nohit" )
-- create chains
for i=1,#offsets_x do
    local offset_x = offsets_x[i]
    local offset_y = offsets_y[i]
    local x = pos_x + offset_x
    local y = pos_y + offset_y

    local ceiling_found,_,ceiling_y = RaytracePlatforms( x, y, x, y - search_dist )
    local dist = y - ceiling_y

    if ceiling_found == true and dist > segment_length then
        -- to local space
        y = offset_y - segment_length
        x = offset_x

        local body_id = i * 100 + 200 -- try to generate ids that don't clash

        -- bottom segment. attach to root with id 100
        EntityAddComponent2( entity_id, "PhysicsImageShapeComponent",
        {
            body_id = body_id,
            offset_x = x,
            offset_y = y,
            image_file = get_random_chain(),
            material = mat
        })
        EntityAddComponent2( entity_id, "PhysicsJoint2Component",
        {
            type = "REVOLUTE_JOINT",
            offset_x = x + 1.5, -- center the joint
            offset_y = y + segment_length, -- joint is on the bottom of the shape
            body1_id = 100,
            body2_id = body_id,
            break_force = break_force_end,
            break_distance = break_distance_end,
        })
        y = y - segment_length

        -- middle segments
        for i2=0, dist / segment_length - 2 do
            body_id = body_id + 1
            EntityAddComponent2( entity_id, "PhysicsImageShapeComponent",
            {
                body_id = body_id,
                offset_x = x,
                offset_y = y,
                image_file = get_random_chain(),
                material = mat
            })
            EntityAddComponent2( entity_id, "PhysicsJoint2Component",
            {
                type = "REVOLUTE_JOINT",
                offset_x = x + 1.5,
                offset_y = y + segment_length,
                body1_id = body_id - 1,
                body2_id = body_id,
                break_force = break_force_mid,
                break_distance = break_distance_mid,
            })
            y = y - segment_length
        end

        -- attach to ceiling
        EntityAddComponent2( entity_id, "PhysicsJoint2Component",
        {
            type = "REVOLUTE_JOINT_ATTACH_TO_NEARBY_SURFACE",
            offset_x = x + 1.5,
            offset_y = y + segment_length,
            body1_id = body_id,
            break_force = break_force_end,
            break_distance = break_distance_end,
            ray_x = 0,
            ray_y = -32,
        })
    end
end

-- done
-- PhysicsBody2InitFromComponents( entity_id )
EntitySetComponentIsEnabled( entity_id, GetUpdatedComponentID(), false)