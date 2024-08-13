local bitser = dofile_once("mods/quant.ew/files/lib/bitser.lua")

local util = {}

function util.string_split( s, splitter )
    local words = {};
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
    return bitser.loads(value)
end

function util.set_ent_variable(entity, key, value)
    local storage = EntityGetFirstComponentIncludingDisabled(entity, "VariableStorageComponent", key)
    ComponentSetValue2(storage, "value_string", bitser.dumps(value))
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
        for i, item in ipairs(items or {}) do
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
    local tags_string = string.match(text, [[tags="(.-)">]])
    print("Tag string ", tags_string)

    local tags = util.string_split(tags_string, ",")
    
    return tags
end)

-- Load an entity that doesn't get saved.
function util.load_ephemerial(path, x, y)
    local entity = EntityCreateNew()
    local ent_2 = EntityLoad(path, x, y)
    EntityAddChild(entity, ent_2)
    return ent_2
end

function util.replace_text_in(filename, pattern, to)
    local initial_text = ModTextFileGetContent(filename)
    local res_text = string.gsub(initial_text, pattern, to)
    ModTextFileSetContent(filename, res_text)
    print("Replaced text in "..filename)
end

-- Gets (or creates, if it doesn't exist) this component
function util.get_or_create_component(entity, component_name, tag)
    local component = EntityGetFirstComponentIncludingDisabled(entity, component_name, tag)
    if component == nil or component == 0 then
        component = EntityAddComponent2(entity, component_name, {
            _tags = tag,
        })
    end
    return component
end

return util
