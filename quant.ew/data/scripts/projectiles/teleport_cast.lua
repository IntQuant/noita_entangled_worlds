dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local pos_x, pos_y = EntityGetTransform(entity_id)

local proj = EntityGetFirstComponentIncludingDisabled(entity_id, "ProjectileComponent")
local shooter
if proj ~= nil then
    shooter = ComponentGetValue(proj, "mWhoShot")
end
local targets
if shooter ~= nil and EntityGetIsAlive(shooter) and EntityHasTag(shooter, "ew_notplayer") then
    targets = EntityGetInRadiusWithTag(pos_x, pos_y, 96, "ew_peer")
else
    targets = EntityGetInRadiusWithTag(pos_x, pos_y, 96, "homing_target")
end

local rndv = pos_x + pos_y
for _, var in ipairs(EntityGetComponent(entity_id, "VariableStorageComponent") or {}) do
    if ComponentGetValue(var, "name") == "ew_transmutation" then
        rndv = ComponentGetValue(var, "value_int")
    end
end
SetRandomSeed(rndv + 46, rndv - 322)

if #targets > 0 then
    local rnd = Random(1, #targets)
    local target_id = targets[rnd]
    local tx, ty = EntityGetFirstHitboxCenter(target_id)
    EntitySetTransform(entity_id, tx, ty)
end
