dofile( "data/scripts/game_helpers.lua" )
dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local pos_x, pos_y = EntityGetTransform( entity_id )

local comps = EntityGetComponent( entity_id, "VariableStorageComponent" )
local biome = BiomeMapGetName( pos_x, pos_y )
local depth = math.floor( pos_y / 512 )
local depth_remainder = pos_y - depth * 512
local old_depth = 0
local status = 0
local memory = ""

local flag_name = "PERK_PICKED_HOMUNCULUS"
local pickup_count = tonumber( GlobalsGetValue( flag_name .. "_PICKUP_COUNT", "0" ) )
local h_limit = 4 + pickup_count

local scomp,dcomp,mcomp

if ( comps ~= nil ) then
    for i,comp in ipairs( comps ) do
        local name = ComponentGetValue2( comp, "name" )
        if ( name == "status" ) then
            status = ComponentGetValue2( comp, "value_int" )
            scomp = comp
        elseif ( name == "latest_depth" ) then
            old_depth = ComponentGetValue2( comp, "value_int" )
            dcomp = comp
        elseif ( name == "memory" ) then
            memory = ComponentGetValue2( comp, "value_string" )
            mcomp = comp
        end
    end
end

if ( status == 0 ) then
    if ( string.find( biome, "holymountain" ) == nil ) and ( string.find( biome, "victoryroom" ) == nil ) then
        local h = EntityGetWithTag( "homunculus" )
        status = 1
        
        if ( #h < h_limit ) then
            local n = EntityLoad( "data/entities/misc/homunculus.xml", pos_x, pos_y )
            EntityAddChild(EntityGetParent(entity_id), n)
            EntityLoad( "data/entities/particles/swarm_poof.xml", pos_x, pos_y )
            GamePrint( "$log_homunculus" )
        end
    end
elseif ( status >= 1 ) then
    if ( ( string.find( biome, "holymountain" ) ~= nil ) or ( string.find( biome, "victoryroom" ) ~= nil ) ) and ( old_depth < depth ) and ( depth_remainder > 150 ) then        
        old_depth = math.max( old_depth, depth )
        status = 0
    end
    
    local memoryid = biome
    
    if ( string.find( memory, "_SPAWN" ) == nil ) and ( string.find( memory, biome ) == nil ) and ( string.find( memory, "holymountain" ) == nil ) and ( string.find( memory, "victoryroom" ) == nil ) then
        status = 1
        
        local h = EntityGetWithTag( "homunculus" )
        if ( #h < h_limit ) then
            local n = EntityLoad( "data/entities/misc/homunculus.xml", pos_x, pos_y )
            EntityAddChild(EntityGetParent(entity_id), n)
            EntityLoad( "data/entities/particles/swarm_poof.xml", pos_x, pos_y )
            GamePrint( "$log_homunculus" )
        end
        memoryid = biome .. "_SPAWN"
    end
    
    if ( #memory == 0 ) then
        memory = memoryid
    else
        local newmemory = ""
        
        local i = 0
        for v in string.gmatch( memory, "%S+" ) do
            i = i + 1
            if ( i == 1 ) and ( v ~= biome ) then
                newmemory = memoryid
            end
            
            if ( i < 4 ) then
                newmemory = newmemory .. " " .. v
            end
        end
        
        memory = newmemory
    end
end

if ( scomp ~= nil ) then
    ComponentSetValue2( scomp, "value_int", status )
end

if ( dcomp ~= nil ) then
    ComponentSetValue2( dcomp, "value_int", old_depth )
end

if ( mcomp ~= nil ) then
    ComponentSetValue2( mcomp, "value_string", memory )
end