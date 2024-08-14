local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")
local rpc = net.new_rpc_namespace()

function player_color(player_entity)
    local cape = nil
    local player_arm = nil

    local player_child_entities = EntityGetAllChildren( player_entity )
    if ( player_child_entities ~= nil ) then
        for _, child_entity in ipairs( player_child_entities ) do
            local child_entity_name = EntityGetName( child_entity )
            if ( child_entity_name == "cape" ) then
                cape = child_entity
            end
            if ( child_entity_name == "arm_r" ) then
                player_arm = child_entity
            end
        end
    end

    local player_sprite_component = EntityGetFirstComponent( player_entity, "SpriteComponent" )
    local player_sprite_file = "mods/quant.ew/files/system/player/tmp/" .. ctx.my_id .. ".xml"
    ComponentSetValue( player_sprite_component, "image_file", player_sprite_file )

    local player_arm_sprite_component = EntityGetFirstComponent( player_arm, "SpriteComponent" )
    local player_arm_sprite_file = "mods/quant.ew/files/system/player/tmp/" .. ctx.my_id .. "_arm.xml"
    ComponentSetValue( player_arm_sprite_component, "image_file", player_arm_sprite_file )

    EntityKill(cape)
    local player_cape_sprite_file = "mods/quant.ew/files/system/player/tmp/" .. ctx.my_id .. "_cape.xml"
    local cape2 = EntityLoad(player_cape_sprite_file, 0, 0)
    EntityAddChild( player_entity, cape2 )
end

function player_cosmetics(player_entity)
    local player_components = EntityGetAllComponents(player_entity)
    if player_components ~= nil then
        for _, comp in ipairs(player_components) do
            if ComponentGetValue2(comp, "image_file") == "data/enemies_gfx/player_amulet.xml" and ModDoesFileExist("mods/quant.ew/files/system/player/tmp/no_amulet") then
                EntitySetComponentIsEnabled(player_entity, comp, false)
            elseif ComponentGetValue2(comp, "image_file") == "data/enemies_gfx/player_amulet_gem.xml" and ModDoesFileExist("mods/quant.ew/files/system/player/tmp/no_amulet_gem") then
                EntitySetComponentIsEnabled(player_entity, comp, false)
            elseif ComponentGetValue2(comp, "image_file") == "data/enemies_gfx/player_hat2.xml" and ModDoesFileExist("mods/quant.ew/files/system/player/tmp/no_crown") then
                EntitySetComponentIsEnabled(player_entity, comp, false)
            end
        end
    end
end

function send_player_cosmetics(id)
    rpc.set_cosmetics_all(id,
            ModDoesFileExist("mods/quant.ew/files/system/player/tmp/no_amulet"),
            ModDoesFileExist("mods/quant.ew/files/system/player/tmp/no_amulet_gem"),
            ModDoesFileExist("mods/quant.ew/files/system/player/tmp/no_crown"))
end

rpc.opts_everywhere()
function rpc.set_cosmetics_all(id, amulet, gem, crown)
    for peer_id, player_data in pairs(ctx.players) do
        if peer_id == id then
            local player_entity = player_data.entity
            local player_components = EntityGetAllComponents(player_entity)
            if player_components ~= nil then
                for _, comp in ipairs(player_components) do
                    if ComponentGetValue2(comp, "image_file") == "data/enemies_gfx/player_amulet.xml" and amulet then
                        EntitySetComponentIsEnabled(player_entity, comp, false)
                    elseif ComponentGetValue2(comp, "image_file") == "data/enemies_gfx/player_amulet_gem.xml" and gem then
                        EntitySetComponentIsEnabled(player_entity, comp, false)
                    elseif ComponentGetValue2(comp, "image_file") == "data/enemies_gfx/player_hat2.xml" and crown then
                        EntitySetComponentIsEnabled(player_entity, comp, false)
                    end
                end
            end
        end
    end
end