local rpc = net.new_rpc_namespace()

local cos = {}

function cos.player_color(player_entity)
    local cape
    local player_arm

    local player_child_entities = EntityGetAllChildren(player_entity)
    if player_child_entities ~= nil then
        for _, child_entity in ipairs(player_child_entities) do
            local child_entity_name = EntityGetName(child_entity)
            if child_entity_name == "cape" then
                cape = child_entity
            end
            if child_entity_name == "arm_r" then
                player_arm = child_entity
            end
        end
    end

    local player_sprite_component = EntityGetFirstComponent(player_entity, "SpriteComponent")
    local damage = EntityGetFirstComponent(player_entity, "DamageModelComponent")
    local player_sprite_component_lukki =
        EntityGetFirstComponentIncludingDisabled(player_entity, "SpriteComponent", "lukki_enable")
    local player_sprite_file = "mods/quant.ew/files/system/player/tmp/" .. ctx.my_id .. ".xml"
    local player_sprite_file_lukki = "mods/quant.ew/files/system/player/tmp/" .. ctx.my_id .. "_lukki.xml"
    local player_arm_sprite_component = EntityGetFirstComponent(player_arm, "SpriteComponent")
    if player_sprite_component == nil or player_arm_sprite_component == nil then
        return
    end
    if damage ~= nil then
        ComponentSetValue(
            damage,
            "ragdoll_filenames_file",
            "mods/quant.ew/files/system/player/tmp/" .. ctx.my_id .. "_ragdoll.txt"
        )
    end
    ComponentSetValue(player_sprite_component, "image_file", player_sprite_file)
    if player_sprite_component_lukki ~= nil then
        ComponentSetValue(player_sprite_component_lukki, "image_file", player_sprite_file_lukki)
    end

    local player_arm_sprite_file = "mods/quant.ew/files/system/player/tmp/" .. ctx.my_id .. "_arm.xml"
    ComponentSetValue(player_arm_sprite_component, "image_file", player_arm_sprite_file)

    EntityKill(cape)
    local player_cape_sprite_file = "mods/quant.ew/files/system/player/tmp/" .. ctx.my_id .. "_cape.xml"
    local x, y = EntityGetTransform(ctx.my_player.entity)
    local cape2 = EntityLoad(player_cape_sprite_file, x, y)
    EntityAddChild(player_entity, cape2)
end

function cos.player_cosmetics(player_entity)
    local player_components = EntityGetAllComponents(player_entity)
    if player_components ~= nil then
        for _, comp in ipairs(player_components) do
            if
                ComponentGetValue2(comp, "image_file") == "data/enemies_gfx/player_amulet.xml"
                and ModDoesFileExist("mods/quant.ew/files/system/player/tmp/no_amulet")
            then
                EntitySetComponentIsEnabled(player_entity, comp, false)
            elseif
                ComponentGetValue2(comp, "image_file") == "data/enemies_gfx/player_amulet_gem.xml"
                and ModDoesFileExist("mods/quant.ew/files/system/player/tmp/no_amulet_gem")
            then
                EntitySetComponentIsEnabled(player_entity, comp, false)
            elseif
                ComponentGetValue2(comp, "image_file") == "data/enemies_gfx/player_hat2.xml"
                and ModDoesFileExist("mods/quant.ew/files/system/player/tmp/no_crown")
            then
                EntitySetComponentIsEnabled(player_entity, comp, false)
            end
        end
    end
end

function cos.send_player_cosmetics(id)
    rpc.set_cosmetics_all(
        id,
        ModDoesFileExist("mods/quant.ew/files/system/player/tmp/no_amulet"),
        ModDoesFileExist("mods/quant.ew/files/system/player/tmp/no_amulet_gem"),
        ModDoesFileExist("mods/quant.ew/files/system/player/tmp/no_crown")
    )
end

local function set_cosmetics(id, amulet, gem, crown)
    local player_entity = ctx.players[id].entity
    local player_components = EntityGetComponent(player_entity, "SpriteComponent", "character")
    if player_components ~= nil then
        for _, comp in ipairs(player_components) do
            if comp == nil then
                goto continue
            end
            if amulet and ComponentGetValue2(comp, "image_file") == "data/enemies_gfx/player_amulet.xml" then
                EntitySetComponentIsEnabled(player_entity, comp, false)
            elseif gem and ComponentGetValue2(comp, "image_file") == "data/enemies_gfx/player_amulet_gem.xml" then
                EntitySetComponentIsEnabled(player_entity, comp, false)
            elseif crown and ComponentGetValue2(comp, "image_file") == "data/enemies_gfx/player_hat2.xml" then
                EntitySetComponentIsEnabled(player_entity, comp, false)
            end
            ::continue::
        end
    end
end

function cos.set_cosmetics_locally(id)
    set_cosmetics(
        id,
        ModDoesFileExist("mods/quant.ew/files/system/player/tmp/no_amulet"),
        ModDoesFileExist("mods/quant.ew/files/system/player/tmp/no_amulet_gem"),
        ModDoesFileExist("mods/quant.ew/files/system/player/tmp/no_crown")
    )
end

rpc.opts_everywhere()
function rpc.set_cosmetics_all(id, amulet, gem, crown)
    set_cosmetics(id, amulet, gem, crown)
end

return cos
