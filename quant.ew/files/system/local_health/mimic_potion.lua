function init(entity_id)
    EntityRemoveTag(entity_id, "enemy") -- to fix bug with RISKY_CRITICAL, which uses this tag to detect enemies nearby
end

function death(damage_type_bit_field, damage_message, entity_thats_responsible, drop_items)
    local entity_id = GetUpdatedEntityID()
    local x, y = EntityGetTransform(entity_id)
    SetRandomSeed(x, y)

    -- see if filled (alive)
end

-- this is needed because item component enable/disable logic doesn't enable the legs for some reason
function enabled_changed(entity_id, is_enabled)
    -- NOTE( Petri ): Bug fixing homing targeting mimic potion in hand.
    -- This section was skipped when is_enabled was false
    if EntityGetParent(entity_id) ~= NULL_ENTITY then
        EntityRemoveTag(entity_id, "homing_target")
    else
        EntityAddTag(entity_id, "homing_target")
    end

    if is_enabled == false then
        return
    end

    local c = EntityGetAllChildren(entity_id)
    if c ~= nil then
        for a, b in ipairs(c) do
            EntitySetComponentsWithTagEnabled(b, "enabled_in_world", true)
        end
    end
end

-- main update
local entity_id = GetUpdatedEntityID()
local x, y = EntityGetTransform(entity_id)
SetRandomSeed(x, y)

-- make pickable if charmed
local comp = GameGetGameEffect(entity_id, "CHARM")
if (comp ~= nil) and (comp ~= NULL_ENTITY) then
    EntitySetComponentsWithTagEnabled(entity_id, "enabled_if_charmed", true)
else
    EntitySetComponentsWithTagEnabled(entity_id, "enabled_if_charmed", false)
end

-- see if filled (alive)
local alive = true

-- make unconscious if empty
local parent = EntityGetParent(entity_id)
if parent == NULL_ENTITY then
    EntitySetComponentsWithTagEnabled(entity_id, "alive", alive)
end

component_write(EntityGetFirstComponent(entity_id, "IKLimbsAnimatorComponent"), { is_limp = (alive == false) })

-- make random noises
if alive and (Random(1, 8) == 1) then
    if Random(1, 4) == 1 then
        GameEntityPlaySound(entity_id, "jump")
    else
        GameEntityPlaySound(entity_id, "damage/projectile")
    end
end

if alive then
    local var = EntityGetFirstComponentIncludingDisabled(entity_id, "VariableStorageComponent", "potion_mimic_awoken")
    if var ~= nil then
        local awoken = ComponentGetValue2(var, "value_bool")
        if not awoken then
            ComponentSetValue2(var, "value_bool", true)
        end
    end
end
