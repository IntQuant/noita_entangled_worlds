ModLuaFileAppend("data/scripts/items/utility_box.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/items/chest_random.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/buildings/chest_steel.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/items/chest_random_super.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/buildings/chest_light.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/scripts/buildings/chest_dark.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")
ModLuaFileAppend("data/biome_impl/static_tile/chest_darkness.lua", "mods/quant.ew/files/resource/cbs/chest_sync.lua")

ModLuaFileAppend("data/scripts/buildings/forge_item_convert.lua", "mods/quant.ew/files/system/forge/append.lua")
util.prepend(
    "data/scripts/buildings/forge_item_convert.lua",
    "local converted = false",
    "local converted = false\n"
        .. 'local util = dofile_once("mods/quant.ew/files/resource/util_min.lua")\n'
        .. "local kill = EntityKill\n"
        .. "local function EntityKill(ent)\n"
        .. 'if EntityHasTag(ent, "broken_wand") then\n'
        .. "local x, y = EntityGetTransform(ent)\n"
        .. 'CrossCall("ew_broken_wand", ent, x, y)\n'
        .. "end\n"
        .. "return kill(ent)\n"
        .. "end\n"
        .. "local first = false\n"
        .. "local rt = EntityGetRootEntity\n"
        .. "local function EntityGetRootEntity(ent)\n"
        .. "if util.do_i_own(ent) then\n"
        .. "return rt(ent)\n"
        .. "else\n"
        .. "if rt(ent) == ent then\n"
        .. "converted = true\n"
        .. "end\n"
        .. "return 0\n"
        .. "end\n"
        .. "end"
)

util.prepend(
    "data/scripts/buildings/sun/spot_1_finish.lua",
    "local converted = false",
    "local converted = false\n"
        .. 'local util = dofile_once("mods/quant.ew/files/resource/util_min.lua")\n'
        .. "local rt = EntityGetRootEntity\n"
        .. "local function EntityGetRootEntity(ent)\n"
        .. "if util.do_i_own(ent) then\n"
        .. "return rt(ent)\n"
        .. "else\n"
        .. "if rt(ent) == ent then\n"
        .. "converted = true\n"
        .. "end\n"
        .. "return 0\n"
        .. "end\n"
        .. "end"
)

local nxml = dofile_once("mods/quant.ew/files/lib/nxml.lua")

local thrown = {}

local dead = {}

local chest = {}

local broken_wands = {}

local gid_chest = {}

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
    --[[if not added_anything then
        -- print("No PhysicsBody2Component to edit in", filename)
    else
        -- print("Updated PhysicsBody2Component in", filename)
    end]]
end

util.add_cross_call("ew_broken_wand", function(ent, x, y)
    local gid
    for _, v in ipairs(EntityGetComponent(ent, "VariableStorageComponent") or {}) do
        if ComponentGetValue2(v, "name") == "ew_gid_lid" then
            gid = v
            break
        end
    end
    if gid ~= nil then
        table.insert(broken_wands, { x, y })
    end
end)

util.add_cross_call("ew_thrown", function(thrown_item)
    if
        thrown_item ~= nil
        and not EntityHasTag(thrown_item, "polymorphed_player")
        and not EntityHasTag(thrown_item, "ew_peer")
        and not EntityHasTag(thrown_item, "ew_client")
    then
        table.insert(thrown, thrown_item)
    end
end)

util.add_cross_call("ew_death_notify", function(entity, wait_on_kill, x, y, file, responsible)
    table.insert(dead, { entity, wait_on_kill, x, y, file, responsible })
end)

util.add_cross_call("ew_chest_opened", function(x, y, rx, ry, file, entity, dont)
    local gid
    for _, v in ipairs(EntityGetComponent(entity, "VariableStorageComponent") or {}) do
        if ComponentGetValue2(v, "name") == "ew_gid_lid" then
            gid = v
            break
        end
    end
    if gid ~= nil then
        gid = ComponentGetValue2(gid, "value_string")
        table.insert(gid_chest, gid)
        if dont == nil then
            table.insert(chest, { x, y, rx, ry, file, gid })
        end
    end
end)

util.add_cross_call("ew_has_opened_chest", function(entity)
    local gid
    for _, v in ipairs(EntityGetComponent(entity, "VariableStorageComponent") or {}) do
        if ComponentGetValue2(v, "name") == "ew_gid_lid" then
            gid = v
            break
        end
    end
    if gid ~= nil then
        return table.contains(gid_chest, ComponentGetValue2(gid, "value_string"))
    else
        return false
    end
end)

local mod = {}

--[[local function bool_to_truefalse(v)
    if v then
        return "true"
    else
        return "false"
    end
end]]

function mod.on_world_update()
    local c_thrown = thrown
    local c_chest = chest
    local wands = broken_wands
    thrown = {}
    chest = {}
    broken_wands = {}
    for _, ent in ipairs(c_thrown) do
        if EntityGetIsAlive(ent) then
            ewext.des_item_thrown(ent)
        end
    end
    for _, data in ipairs(c_chest) do
        ewext.des_chest_opened(data[1], data[2], data[3], data[4], data[5], data[6])
    end
    for _, data in ipairs(wands) do
        ewext.des_broken_wand(data[1], data[2])
    end
end

function mod.on_world_update_post()
    local c_dead = dead
    dead = {}
    for _, data in ipairs(c_dead) do
        ewext.des_death_notify(data[1], data[2], data[3], data[4], data[5], data[6])
    end
end

return mod
