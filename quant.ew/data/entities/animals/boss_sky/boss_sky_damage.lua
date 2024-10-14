dofile_once("data/scripts/lib/utilities.lua")

local entity_id    = GetUpdatedEntityID()
local x, y = EntityGetTransform( GetUpdatedEntityID() )

local var = EntityGetFirstComponent( entity_id, "VariableStorageComponent" )
local hp_percentage = 0
if var ~= nil then
    hp_percentage = ComponentGetValue2( var, "value_float" )
end


local players = get_players()
if #players == 0 then
    players = EntityGetWithTag("polymorphed_player")
    for i = #players, 1, -1 do
        local ent = players[i]
        if EntityHasTag(ent, "ew_notplayer") then
            table.remove(players, i)
        end
    end
    if #players > 0 then
        local lifetime = EntityGetFirstComponent( entity_id, "LifetimeComponent" )
        if lifetime ~= nil then
            ComponentSetValue2( lifetime, "kill_frame", GameGetFrameNum() + 90 )
            return
        end
    end
end

if #players > 0 then
    local player_id = players[1]
    local px, py, ang, sx, sy = EntityGetTransform( player_id  )

    local hx, hy = EntityGetHotspot( player_id, "cape_root", false )
    local tx, ty = px+hx, py+hy-2
    EntitySetTransform( entity_id, tx, ty )

    -- holy damage already has 1.5x multiplier
    component_read( EntityGetFirstComponent( player_id, "DamageModelComponent" ), { max_hp = 0, hp = 0 }, function(comp)
        local player_hp_percentage = comp.hp / comp.max_hp
        if player_hp_percentage >= hp_percentage then
            local damage = math.max( 0 , comp.max_hp * ( player_hp_percentage - hp_percentage ) );
            if damage > 0 and not EntityHasTag(player_id, "ew_notplayer") then
                EntityInflictDamage( player_id, damage, "DAMAGE_HOLY", "$damage_holy", "NONE", 0, 0, NULL_ENTITY )
                EntityLoad( "data/entities/particles/poof_red_tiny.xml", tx, ty )
            end
        end
    end)
end