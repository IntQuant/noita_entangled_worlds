dofile_once("data/scripts/lib/utilities.lua")

local max_range = 150
local min_range = 50

local entity_id = GetUpdatedEntityID()
local pos_x, pos_y = EntityGetTransform(entity_id)

local function start_chase()
    -- enable chase & cell eater, disable this script
    print("start chase")
    EntitySetComponentsWithTagEnabled(entity_id, "enable_when_player_seen", true)
    EntityRemoveComponent(entity_id, GetUpdatedComponentID())
end

local target = EntityGetClosestWithTag(pos_x, pos_y, "ew_peer")
if not target or target == 0 then return end

local tx, ty = EntityGetTransform(target)
local dist = get_distance(tx, ty, pos_x, pos_y)
if dist > max_range then return end -- too far
if dist < min_range then
    -- very near
    start_chase()
    return
end

-- check los
local did_hit = RaytraceSurfaces(pos_x, pos_y, tx, ty)
if not did_hit then start_chase() end