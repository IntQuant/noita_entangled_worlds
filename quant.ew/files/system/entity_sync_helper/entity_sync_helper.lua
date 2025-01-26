ModLuaFileAppend("data/scripts/items/utility_box.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/items/chest_random.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/buildings/chest_steel.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/items/chest_random_super.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/buildings/chest_light.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/buildings/chest_dark.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/biome_impl/static_tile/chest_darkness.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")

local nxml = dofile_once("mods/quant.ew/files/lib/nxml.lua")

local thrown = {}

local dead = {}

local chest = {}

-- Add extra entities to entity sync
for filename, _ in pairs(constants.phys_sync_allowed) do
    util.add_tag_to(filename, "ew_synced")

    local added_anything = false
    for content in nxml.edit_file(filename) do
        for elem in content:each_of("PhysicsBody2Component") do
            elem:set("destroy_body_if_entity_destroyed", true)
            elem:set("kill_entity_after_initialized", false)
            added_anything = true
        end
    end
    if not added_anything then
        -- print("No PhysicsBody2Component to edit in", filename)
    else
        -- print("Updated PhysicsBody2Component in", filename)
    end
end

util.add_cross_call("ew_thrown", function(thrown_item)
    if thrown_item ~= nil then
        table.insert(thrown, thrown_item)
    end
end)

util.add_cross_call("ew_death_notify", function(entity, wait_on_kill, drops_gold, x, y, file, responsible)
    table.insert(dead, { entity, wait_on_kill, drops_gold, x, y, file, responsible })
end)

util.add_cross_call("ew_chest_opened", function(x, y, rx, ry, file, entity)
    table.insert(chest, { x, y, rx, ry, file, entity })
end)

local mod = {}

function mod.on_world_update_post()
    for _, ent in ipairs(thrown) do
        ewext.des_item_thrown(ent)
    end
    for _, data in ipairs(dead) do
        ewext.des_death_notify(data[1], data[2], data[3], data[4], data[5], data[6], data[7])
    end
    for _, data in ipairs(chest) do
        ewext.des_chest_opened(data[1], data[2], data[3], data[4], data[5], data[6])
    end
    thrown = {}
    dead = {}
    chest = {}
end

return mod
