dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local x, y = EntityGetTransform(entity_id)

local projectiles = EntityGetWithTag("projectile")

if #projectiles > 0 then
    for i, projectile_id in ipairs(projectiles) do
        local px, py = EntityGetTransform(projectile_id)

        local distance = math.abs(x - px) + math.abs(y - py)

        if (distance < 64) and (entity_id ~= projectile_id) then
            distance = math.sqrt((x - px) ^ 2 + (y - py) ^ 2)

            if distance < 48 then
                local projectilecomponents = EntityGetComponent(projectile_id, "ProjectileComponent")

                if projectilecomponents ~= nil then
                    for j, comp_id in ipairs(projectilecomponents) do
                        ComponentSetValue(comp_id, "on_death_explode", "0")
                        ComponentSetValue(comp_id, "on_lifetime_out_explode", "0")
                    end
                end

                SetRandomSeed(px + 325, py - 235)
                local opts = { "duck", "sheep", "sheep_bat", "sheep_fly" }
                local rnd = Random(1, #opts)
                local opt = opts[rnd]

                if CrossCall("ew_do_i_own", entity_id) then
                    EntityLoad("data/entities/animals/" .. opt .. ".xml", px, py)
                end
                EntityLoad("data/entities/particles/image_emitters/transmutation_effect.xml", px, py)
                EntityKill(projectile_id)
            end
        end
    end
end
