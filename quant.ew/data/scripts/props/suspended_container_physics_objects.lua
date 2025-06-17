dofile_once("data/scripts/lib/utilities.lua")

local entity_id = GetUpdatedEntityID()
local pos_x, pos_y = EntityGetTransform(entity_id)
pos_x = pos_x - 5

local stones = {
    "data/entities/props/physics_stone_01.xml",
    "data/entities/props/physics_stone_02.xml",
    "data/entities/props/physics_stone_03.xml",
    "data/entities/props/physics_stone_03.xml",
    "data/entities/props/physics_stone_04.xml",
    "data/entities/props/physics_stone_04.xml",
}

local props = {
    "data/entities/props/physics_box_explosive.xml",
    "data/entities/props/physics_barrel_oil.xml",
    "data/entities/props/physics_seamine.xml",
    "data/entities/props/physics/minecart.xml",
}

local count = ProceduralRandomi(pos_x, pos_y, 2, 7)

if not GameHasFlagRun("ew_flag_this_is_host") then
    return
end

for i = 1, count do
    local obj
    local r = ProceduralRandomf(i + pos_x, pos_y + 4)
    if r > 0.9 then
        obj = props[ProceduralRandomi(pos_x - 4, pos_y + i, 1, #props)]
    else
        obj = stones[ProceduralRandomi(pos_x - 4, pos_y + i, 1, #stones)]
    end

    EntityLoad(obj, pos_x + r * 8, pos_y)
    pos_y = pos_y - 5
end
