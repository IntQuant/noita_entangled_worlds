dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local boss_id = EntityGetRootEntity( entity_id )
local x,y = EntityGetTransform( boss_id )

local comp = EntityGetFirstComponent( entity_id, "VariableStorageComponent", "wizard_orb_id" )
if ( comp ~= nil ) then
	local id = ComponentGetValue2( comp, "value_int" )

	local count = 8
	local circle = math.pi * 2
	local inc = circle / count

	local dir = inc * id + CrossCall("ew_host_frame_num") * 0.01

	local nx = x + math.cos( dir ) * 50
	local ny = y - math.sin( dir ) * 50 - 20

	EntitySetTransform( entity_id, nx, ny )
end