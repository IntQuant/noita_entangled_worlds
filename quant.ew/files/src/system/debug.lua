local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local util = dofile_once("mods/quant.ew/files/src/util.lua")

local module = {}

function module.on_local_player_spawn(my_player)
    
    local player_entity = my_player.entity
    -- ~Portal to lab
    -- EntitySetTransform(player_entity, 0, 12600)
    -- The vault
    -- EntitySetTransform(player_entity, 0, 8600-20)

    -- EntitySetTransform(player_entity, -1990, 2171)

    -- EntitySetTransform(player_entity, 512*3+10, 512*3+10)

    -- Kolmi room
    -- EntitySetTransform(player_entity, 3400, 13040)
    
    -- The work
    -- EntitySetTransform(player_entity, 6300, 15155)

    EntitySetTransform(player_entity, 1333, 770)

    -- util.load_ephemerial("mods/quant.ew/files/entities/client.xml", 512*3+20, 512*3+10)
    -- EntityLoad("mods/quant.ew/files/entities/client.xml", 512*3+20, 512*3+10)

    ctx.cap.health.set_max_health(1000)
    ctx.cap.health.set_health(1000)
    -- util.set_ent_health(player_entity, {1000, 1000})
    local wallet = EntityGetFirstComponentIncludingDisabled(player_entity, "WalletComponent")
    ComponentSetValue2(wallet, "money", 100000)
    -- GameSetCameraFree(true)

    -- dofile_once("data/scripts/perks/perk.lua")
    -- local x, y = EntityGetFirstHitboxCenter(player_entity)
    -- perk_spawn(x, y, "LASER_AIM", true)
    -- perk_spawn(x-50, y, "GLASS_CANNON", true)
    -- perk_spawn(x-25, y, "EDIT_WANDS_EVERYWHERE", true)
    -- EntityLoad("data/entities/items/pickup/heart.xml", x-75, y-20)

end

function module.on_world_update_host()
    local hp, max_hp = util.get_ent_health(ctx.my_player.entity)
    if hp < max_hp / 2 then
        -- util.set_ent_health(ctx.my_player.entity, {max_hp, max_hp})
    end
end


return module