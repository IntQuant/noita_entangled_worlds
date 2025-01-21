ModLuaFileAppend("data/scripts/items/utility_box.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/items/chest_random.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/buildings/chest_steel.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/items/chest_random_super.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/buildings/chest_light.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/buildings/chest_dark.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/biome_impl/static_tile/chest_darkness.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")

local nxml = dofile_once("mods/quant.ew/files/lib/nxml.lua")

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
        ewext.des_item_thrown(thrown_item)
    end
end)

util.add_cross_call("ew_death_notify", function(entity, responsible)
    ewext.des_death_notify(entity, responsible)
end)

util.add_cross_call("ew_chest_opened", function(entity)
    ewext.des_chest_opened(entity)
end)

return {}
