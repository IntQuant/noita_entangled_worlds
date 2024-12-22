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
        print("No PhysicsBody2Component to edit in", filename)
    else
        print("Updated PhysicsBody2Component in", filename)
    end
end

return {}
