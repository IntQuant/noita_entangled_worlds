local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")

ModLuaFileAppend("data/scripts/misc/no_heal_in_meat_biome.lua", "mods/quant.ew/files/system/patch_meat_biome/append/biome_check.lua")

local is_in_meat_current = false

local module = {}

function module.on_client_spawned(peer_id, player_data)
    local eff = EntityLoad("data/entities/misc/effect_no_heal_in_meat_biome.xml", 0, 0)
    EntityAddChild(player_data.entity, eff)
end

function module.on_world_update_host()
    if is_in_meat_current then
        local current_health = ctx.cap.health.health()
        local current_limit = tonumber(GlobalsGetValue("ew_meat_health_limit", tostring(current_health)))
        if current_health > current_limit then
            current_health = current_limit
            ctx.cap.health.set_health(current_health)
        else
            current_limit = current_health
            GlobalsSetValue("ew_meat_health_limit", tostring(current_limit))
        end
    end
end

function module.on_world_update()
    if GameGetFrameNum() % 60 ~= 55 then
        return
    end

    local is_boss_dead = GlobalsGetValue( "BOSS_MEAT_DEAD", "0" ) == "1"
    local anyone_in_meat = #(EntityGetWithTag("ew_no_heal_in_meat_biome") or {}) > 0
    anyone_in_meat = anyone_in_meat and not is_boss_dead

    if anyone_in_meat ~= is_in_meat_current then
        if anyone_in_meat then
            GamePrint("The air feels heavier...")
        else
            GamePrint("The air feels normal")
        end
    end
    is_in_meat_current = anyone_in_meat
end

return module