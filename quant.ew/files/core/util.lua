local bitser = dofile_once("mods/quant.ew/files/lib/bitser.lua")
local ffi = require("ffi")
local base64 = dofile_once("mods/quant.ew/files/resource/base64.lua")

local util = {}

local entity_load_orig = EntityLoad

function EntityLoad(path, ...)
    if path == "??SAV/world_state.xml" then
        error("Attempted to EntityLoad WSE")
    end
    return entity_load_orig(path, ...)
end

function util.string_split( s, splitter )
    local words = {};
    if s == nil or splitter == nil then
        return {}
    end
    for word in string.gmatch( s, '([^'..splitter..']+)') do
        table.insert( words, word );
    end
    return words;
end

function util.print_error(error)
    local lines = util.string_split(error, "\n")
    print("---err start---")
    for _, line in ipairs(lines) do
        GamePrint(line)
        print(line)
    end
    print("---err end---")
end

function util.tpcall(fn, ...)
    local res = {xpcall(fn, debug.traceback, ...)}
    if not res[1] then
        util.print_error(res[2])
    end
    return unpack(res)
end

function util.print_traceback()
    util.print_error(debug.traceback())
end

function util.get_ent_variable(entity, key)
    local storage = EntityGetFirstComponentIncludingDisabled(entity, "VariableStorageComponent", key)
    if storage == nil then
        return nil
    end
    local value = ComponentGetValue2(storage, "value_string")
    if value == "" then
        return nil
    end
    return bitser.loads(base64.decode(value))
end

function util.set_ent_variable(entity, key, value)
    local storage = EntityGetFirstComponentIncludingDisabled(entity, "VariableStorageComponent", key)
    ComponentSetValue2(storage, "value_string", base64.encode(bitser.dumps(value)))
end

function util.get_ent_health(entity)
    local damage_model = EntityGetFirstComponentIncludingDisabled(entity, "DamageModelComponent")
    if damage_model == nil then
        return 0, 0, false
    end
    local hp = ComponentGetValue2(damage_model, "hp")
    local max_hp = ComponentGetValue2(damage_model, "max_hp")
    return hp, max_hp, true
end

function util.get_ent_air(entity)
    local damage_model = EntityGetFirstComponentIncludingDisabled(entity, "DamageModelComponent")
    if damage_model == nil then
        return 0, 0
    end
    local air = ComponentGetValue2(damage_model, "air_in_lungs")
    local max_air = ComponentGetValue2(damage_model, "air_in_lungs_max")
    return air, max_air
end

function util.set_ent_health(entity, hp_data)
    local damage_model = EntityGetFirstComponentIncludingDisabled(entity, "DamageModelComponent")
    if damage_model == nil then
        return
    end
    if hp_data[1] ~= nil then
        ComponentSetValue2(damage_model, "hp", hp_data[1])
    end
    if hp_data[2] ~= nil then
        ComponentSetValue2(damage_model, "max_hp", hp_data[2])
    end
end

function util.set_ent_air(entity, air_data)
    local damage_model = EntityGetFirstComponentIncludingDisabled(entity, "DamageModelComponent")
    if damage_model == nil then
        return
    end
    if air_data[1] ~= nil then
        ComponentSetValue2(damage_model, "air_in_lungs", air_data[1])
    end
    if air_data[2] ~= nil then
        ComponentSetValue2(damage_model, "air_in_lungs_max", air_data[2])
    end
end

function util.get_ent_health_cap(entity)
    local damage_model = EntityGetFirstComponentIncludingDisabled(entity, "DamageModelComponent")
    if damage_model == nil then
        return 0
    end
    local cap = ComponentGetValue2(damage_model, "max_hp_cap")
    return cap
end

function util.set_ent_health_cap(entity, cap)
    local damage_model = EntityGetFirstComponentIncludingDisabled(entity, "DamageModelComponent")
    if damage_model == nil then
        return 0
    end
    ComponentSetValue2(damage_model, "max_hp_cap", cap)
end

function util.lerp(a, b, alpha)
    return a * alpha + b * (1 - alpha)
end

function util.set_ent_firing_blocked(entity, do_block)
    local now = GameGetFrameNum();
    local inventory2Comp = EntityGetFirstComponentIncludingDisabled(entity, "Inventory2Component")
    if(inventory2Comp ~= nil)then
        local items = GameGetAllInventoryItems(entity)
        for _, item in ipairs(items or {}) do
            local ability = EntityGetFirstComponentIncludingDisabled( item, "AbilityComponent" );
            if ability then
                if(do_block)then
                    ComponentSetValue2( ability, "mReloadFramesLeft", 2000000 );
                    ComponentSetValue2( ability, "mNextFrameUsable", now + 2000000 );
                    ComponentSetValue2( ability, "mReloadNextFrameUsable", now + 2000000 );

                else
                    ComponentSetValue2( ability, "mReloadFramesLeft", 0 );
                    ComponentSetValue2( ability, "mNextFrameUsable", now );
                    ComponentSetValue2( ability, "mReloadNextFrameUsable", now );
                end
            end
        end
    end
end

-- Adds this component with given data if it doesn't exist
function util.ensure_component_present(entity, component, tag, data, tags)
    local current = EntityGetFirstComponentIncludingDisabled(entity, component, tag)
    data._tags=tags or tag
    if current == nil then
        EntityAddComponent2(entity, component, data)
    end
end

-- Caches function's results by first argument
function util.cached_fn(fn)
    local cache = {}
    function cached(arg, ...)
        if cache[arg] ~= nil then
            return cache[arg]
        end
        local result = fn(arg, ...)
        cache[arg] = result
        return result
    end
    return cached
end

util.load_ents_tags = util.cached_fn(function(path)
    local text = ModTextFileGetContent(path)
    local tags_string = string.match(text, [[tags="(.-)"]])

    -- print("Path", path, "Tags string: ", tostring(tags_string))
    if tags_string == nil then
        return {}
    end

    local tags = util.string_split(tags_string, ",")

    return tags
end)

function util.make_ephemerial(ent)
    if ent ~= nil and EntityGetIsAlive(ent) then
        ewext.make_ephemerial(ent)
    end
end

-- Load an entity that doesn't get saved.
function util.load_ephemerial(path, x, y)
    --local entity = EntityCreateNew()
    --EntityAddTag(entity, "ew_synced_entity")
    --EntitySetTransform(entity, x, y)
    local ent_2 = EntityLoad(path, x, y)
    --EntityAddChild(entity, ent_2)
    util.make_ephemerial(ent_2)
    return ent_2
end

function util.replace_text_in(filename, pattern, to)
    local initial_text = ModTextFileGetContent(filename)
    local res_text = string.gsub(initial_text, pattern, to)
    if initial_text ~= res_text then
        ModTextFileSetContent(filename, res_text)
        print(" Replaced text in "..filename)
    end
end

-- Gets (or creates, if it doesn't exist) this component
function util.get_or_create_component(entity, component_name, tag)
    local component
    if tag == nil then
        component = EntityGetFirstComponentIncludingDisabled(entity, component_name)
    else
        component = EntityGetFirstComponentIncludingDisabled(entity, component_name, tag)
    end
    if component == nil or component == 0 then
        component = EntityAddComponent2(entity, component_name, {
            _tags = tag,
        })
    end
    return component
end

-- Add a tag to a entity xml if it isn't present yet.
function util.add_tag_to(filename, tag)
    local current_tags = util.load_ents_tags(filename)
    if table.contains(current_tags, tag) then
        return
    end
    -- Tag list is cached, update it.
    table.insert(current_tags, tag)


    print(" Adding tag "..tag.." to "..filename)
    local content = ModTextFileGetContent(filename)
    content = string.gsub(content, "Entity(.-)>", function(inner)
        local changed_tags = false
        inner = string.gsub(inner, [[tags="(.-)"]], function(tags)
            changed_tags = true
            return 'tags="'..tags..","..tag..'"'
        end)
        if not changed_tags then
            inner = inner..' tags="'..tag..'"'
        end
        return "Entity "..inner..">"
    end, 1)
    ModTextFileSetContent(filename, content)
end

function util.copy_file_content(from, to)
    ModTextFileSetContent(to, ModTextFileGetContent(from))
end

local type_counter = 0

-- Generates struct types that are generally much more efficient to send.
function util.make_type(typedata)
    local name = "U"..type_counter
    type_counter = type_counter + 1

    local inner = ""

    for _, var in ipairs(typedata.f32 or {}) do
        inner = inner .. "float "..var..";\n"
    end

    for _, var in ipairs(typedata.u8 or {}) do
        inner = inner .. "unsigned char "..var..";\n"
    end

    for _, var in ipairs(typedata.u32 or {}) do
        inner = inner .. "unsigned int "..var..";\n"
    end

    for _, var in ipairs(typedata.bool or {}) do
        inner = inner .. "bool "..var..";\n"
    end

    for _, var in ipairs(typedata.vecbool or {}) do
        inner = inner .. "bool "..var.."[16];\n"
    end

    for _, var in ipairs(typedata.peer_id or {}) do
        inner = inner .. "char "..var.."[16];\n"
    end

    ffi.cdef([[
    #pragma pack(push, 1)
    typedef struct ]] .. name .. [[{
        ]] .. inner .. [[
    } ]] .. name .. [[;
    #pragma pack(pop)
    ]])
    local typ = ffi.typeof(name);
    return typ
end

function util.log(...)
    if ctx.proxy_opt.debug then
        GamePrint(...)
    end
    print(...)
end

function util.serialize_entity(ent)
    -- Serialized entities usually get sent to other clients, and it's a very bad idea to try and send them another WorldState.
    if util.is_world_state_entity_like(ent) then
        error("Tried to serialize WorldStateEntity")
    end
    return np.SerializeEntity(ent)
end

function util.is_world_state_entity_like(ent)
    return EntityHasTag(ent, "world_state") or EntityGetFirstComponentIncludingDisabled(ent, "WorldStateComponent") ~= nil
end

function util.deserialize_entity(ent_data, x, y)
    local ent = EntityCreateNew()
    if x == nil or y == nil then
        np.DeserializeEntity(ent, ent_data)
    else
        np.DeserializeEntity(ent, ent_data, x, y)
    end
    if EntityGetFirstComponentIncludingDisabled(ent, "WorldStateComponent") ~= nil then
        print("Tried to deserialize WorldStateEntity. The world is screwed.")
        EntityKill(ent)
    end
    return ent
end

local cross_calls = {}

function util.add_cross_call(name, fn)
    np.CrossCallAdd(name, fn)
    cross_calls[name] = fn
end

function CrossCall(name, ...)
    cross_calls[name](...)
end

util.add_cross_call("ew_host_frame_num", function()
    if ctx.my_id == ctx.host_id then
        return GameGetFrameNum()
    else
        return ctx.host_frame_num
    end
end)

local FULL_TURN = math.pi * 2

local PhysData = util.make_type({
    f32 = {"x", "y", "vx", "vy", "vr"},
    -- We should be able to cram rotation into 1 byte.
    u8 = {"r"}
})

-- Variant of PhysData for when we don't have any motion.
local PhysDataNoMotion = util.make_type({
    f32 = {"x", "y"},
    -- We should be able to cram rotation into 1 byte.
    u8 = {"r"}
})

local function serialize_phys_component(phys_component)
    local px, py, pr, pvx, pvy, pvr = np.PhysBodyGetTransform(phys_component)
    px, py = PhysicsPosToGamePos(px, py)
    if math.abs(pvx) < 0.01 and math.abs(pvy) < 0.01 and math.abs(pvr) < 0.01 then
        return PhysDataNoMotion {
            x = px,
            y = py,
            r = math.floor((pr % FULL_TURN) / FULL_TURN * 255),
        }
    else
        return PhysData {
            x = px,
            y = py,
            r = math.floor((pr % FULL_TURN) / FULL_TURN * 255),
            vx = pvx,
            vy = pvy,
            vr = pvr,
        }
    end
end

local function deserialize_phys_component(phys_component, phys_info, fps)
    local x, y = GamePosToPhysicsPos(phys_info.x, phys_info.y)
    if ffi.typeof(phys_info) == PhysDataNoMotion then
        np.PhysBodySetTransform(phys_component, x, y, phys_info.r / 255 * FULL_TURN, 0, 0, 0)
    else
        local m = fps / ctx.my_player.fps
        np.PhysBodySetTransform(phys_component, x, y, phys_info.r / 255 * FULL_TURN, phys_info.vx * m, phys_info.vy * m, phys_info.vr * m)
    end
end

function util.get_phys_info(entity, kill)
    local phys_info = {}
    local phys_info_2 = {}
    for _, phys_component in ipairs(EntityGetComponent(entity, "PhysicsBodyComponent") or {}) do
        if phys_component ~= nil and phys_component ~= 0 then
            local ret, info = pcall(serialize_phys_component, phys_component)
            if not ret and kill then
                EntityKill(entity)
                return {{}, {}}
            end
            table.insert(phys_info, info)
        end
    end

    for _, phys_component in ipairs(EntityGetComponent(entity, "PhysicsBody2Component") or {}) do
        if phys_component ~= nil and phys_component ~= 0 then
            local initialized = ComponentGetValue2(phys_component, "mInitialized")
            if initialized then
                local ret, info = pcall(serialize_phys_component, phys_component)
                if not ret and kill then
                    EntityKill(entity)
                    return {{}, {}}
                end
                table.insert(phys_info_2, info)
            else
                table.insert(phys_info_2, nil)
            end
        end
    end
    return {phys_info, phys_info_2}
end

function util.set_phys_info(entity, data, fps)
    local phys_infos, phys_infos_2 = data[1], data[2]
    local has_set = false
    for i, phys_component in ipairs(EntityGetComponent(entity, "PhysicsBodyComponent") or {}) do
        local phys_info = phys_infos[i]
        if phys_component ~= nil and phys_component ~= 0 and phys_info ~= nil
                and ComponentGetValue2(phys_component, "mPixelCount") ~= 0 then
            deserialize_phys_component(phys_component, phys_info, fps)
            has_set = true
        end
    end
    for i, phys_component in ipairs(EntityGetComponent(entity, "PhysicsBody2Component") or {}) do
        local phys_info = phys_infos_2[i]
        if phys_component ~= nil and phys_component ~= 0 and phys_info ~= nil
                and ComponentGetValue2(phys_component, "mPixelCount") ~= 0
                -- A physics body doesn't exist otherwise, causing a crash
                and ComponentGetValue2(phys_component, "mInitialized") then
            deserialize_phys_component(phys_component, phys_info, fps)
            has_set = true
        end
    end
    return has_set
end

return util