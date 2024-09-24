local wandfinder = {}

local function entity_is_wand(entity_id)
    local ability_component = EntityGetFirstComponentIncludingDisabled(entity_id, "AbilityComponent")
    if ability_component == nil then return false end
    return ComponentGetValue2(ability_component, "use_gun_script") == true
end

local function get_all_wands(dont_do)
    local wands = {}
    local items = GameGetAllInventoryItems(ctx.my_player.entity) or {}
    for _, item in ipairs(items) do
        local use = true
        for _, item2 in ipairs(dont_do) do
            if item == item2 then
                use = false
            end
        end
        if entity_is_wand(item) and use then
            table.insert(wands, item)
        end
    end
    return wands
end

function wandfinder.find_attack_wand(dont_do)
    local wands = get_all_wands(dont_do)
    if #wands == 0 then
        return nil
    end
    local largest = {-1, -1}
    for entity, tuple in pairs(ctx.my_player.wand_fire_count) do
        local fire_count, _ = tuple[1], tuple[2]
        for _, wand in ipairs(wands) do
            if wand == entity then
                goto here
            end
        end
        goto continue
        ::here::
        if largest[2] < fire_count then
            largest = {entity, fire_count}
        end
        ::continue::
    end
    if largest[1] == -1 then
        return wands[Random(1, #wands)]
    end
    return largest[1]
end

function wandfinder.set_wands_after_poly()
    local wands = get_all_wands({})
    for entity, tuple in pairs(ctx.my_player.wand_fire_count) do
        local _, slot = tuple[1], tuple[2]
        for _, wand in ipairs(wands) do
            local item = EntityGetFirstComponentIncludingDisabled(wand, "ItemComponent")
            local slot2 = ComponentGetValue2(item, "inventory_slot")
            if slot == slot2 then
                ctx.my_player.wand_fire_count[wand] = ctx.my_player.wand_fire_count[entity]
                table.remove(ctx.my_player.wand_fire_count, entity)
                break
            end
        end
    end
end

return wandfinder