dofile_once("data/scripts/lib/utilities.lua")
dofile_once("data/scripts/gun/procedural/gun_action_utils.lua")

function get_random_from(target)
    local rnd = Random(1, #target)

    return tostring(target[rnd])
end

function get_multiple_random_from(target, amount_)
    local amount = amount_ or 1

    local result = {}

    for i = 1, amount do
        local rnd = Random(1, #target)

        table.insert(result, tostring(target[rnd]))
    end

    return result
end

function get_random_between_range(target)
    local minval = target[1]
    local maxval = target[2]

    return Random(minval, maxval)
end

local entity_id = GetUpdatedEntityID()
local x, y = EntityGetTransform(entity_id)
local sx, sy = CrossCall("ew_per_peer_seed")
SetRandomSeed(x + 19 - sx, y - 19 + sy)

local ability_comp = EntityGetFirstComponent(entity_id, "AbilityComponent")

local gun = {}
gun.name = { "Bomb wand" }
gun.deck_capacity = 1
gun.actions_per_round = 1
gun.reload_time = { 1, 10 }
gun.shuffle_deck_when_empty = 1
gun.fire_rate_wait = { 3, 8 }
gun.spread_degrees = 0
gun.speed_multiplier = 1
gun.mana_charge_speed = { 5, 20 }
gun.mana_max = { 80, 110 }
gun.actions = { "BOMB", "DYNAMITE", "MINE", "ROCKET", "GRENADE" }

local mana_max = get_random_between_range(gun.mana_max)
local deck_capacity = gun.deck_capacity

ComponentSetValue(ability_comp, "ui_name", get_random_from(gun.name))

ComponentObjectSetValue(ability_comp, "gun_config", "reload_time", get_random_between_range(gun.reload_time))
ComponentObjectSetValue(
    ability_comp,
    "gunaction_config",
    "fire_rate_wait",
    get_random_between_range(gun.fire_rate_wait)
)
ComponentSetValue(ability_comp, "mana_charge_speed", get_random_between_range(gun.mana_charge_speed))

ComponentObjectSetValue(ability_comp, "gun_config", "actions_per_round", gun.actions_per_round)
ComponentObjectSetValue(ability_comp, "gun_config", "deck_capacity", deck_capacity)
ComponentObjectSetValue(ability_comp, "gun_config", "shuffle_deck_when_empty", gun.shuffle_deck_when_empty)
ComponentObjectSetValue(ability_comp, "gunaction_config", "spread_degrees", gun.spread_degrees)
ComponentObjectSetValue(ability_comp, "gunaction_config", "speed_multiplier", gun.speed_multiplier)

ComponentSetValue(ability_comp, "mana_max", mana_max)
ComponentSetValue(ability_comp, "mana", mana_max)

local action_count = 1

local gun_action = "BOMB"
local n_of_deaths = tonumber(StatsGlobalGetValue("death_count"))

if n_of_deaths >= 1 then
    if Random(1, 100) < 50 then
        gun_action = get_random_from(gun.actions)
    end
end

for i = 1, action_count do
    --AddGunActionPermanent( entity_id, gun_action )
    AddGunAction(entity_id, gun_action)
end
