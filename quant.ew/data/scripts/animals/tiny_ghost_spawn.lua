dofile_once("data/scripts/lib/utilities.lua")

--function collision_trigger(target_id)
local entity_id = GetUpdatedEntityID()
local x, y = EntityGetTransform(entity_id)
local target_id = EntityGetClosestWithTag(x, y, "enemy")
if not target_id then
    return
end

if not IsPlayer(target_id) and EntityGetIsAlive(target_id) then
    --print("ghost added: " .. x .. ", " .. y)

    local children = EntityGetAllChildren(entity_id)
    if #children == 0 then
        return
    end
    local ghost_id = children[1]

    -- reduce health of target for balance
    component_readwrite(
        EntityGetFirstComponent(target_id, "DamageModelComponent"),
        { hp = 0, max_hp = 0 },
        function(comp)
            comp.max_hp = math.max(comp.max_hp * 0.75, comp.max_hp - 3)
            comp.hp = comp.max_hp
        end
    )

    -- enable ghost
    for _, comp in pairs(EntityGetAllComponents(ghost_id)) do
        EntitySetComponentIsEnabled(ghost_id, comp, true)
    end

    -- transfer ghost & remove spawner
    EntityRemoveFromParent(ghost_id)
    EntityAddChild(target_id, ghost_id)
    EntityKill(entity_id)
end
