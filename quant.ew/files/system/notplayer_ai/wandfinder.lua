local wandfinder = {}

local function entity_is_wand(entity_id)
    local ability_component = EntityGetFirstComponentIncludingDisabled(entity_id, "AbilityComponent")
    if ability_component == nil then return false end
    return ComponentGetValue2(ability_component, "use_gun_script") == true
end

local function get_all_wands()
    local wands = {}
    local items = GameGetAllInventoryItems(ctx.my_player.entity) or {}
    for _, item in ipairs(items) do
        if entity_is_wand(item) then
            table.insert(wands, item)
        end
    end
    return wands
end

function wandfinder.find_attack_wand()
    local wands = get_all_wands()
    if #wands == 0 then
        return nil
    end
    local largest = {-1, -1}
    for entity, fire_count in pairs(ctx.my_player.wand_fire_count) do
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
    if largest == {-1, -1} then
        return wands[1]
    end
    return largest[1]
end

return wandfinder