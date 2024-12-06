local util = dofile_once("mods/quant.ew/files/core/util.lua")

local function lazyload()
    dofile_once("data/scripts/perks/perk_list.lua")
end

local perk_fns = {}

-- Which perks we do not add to clients.
local perks_to_ignore = {
    GAMBLE = true,
    PERKS_LOTTERY = true,
    REMOVE_FOG_OF_WAR = true,
    MEGA_BEAM_STONE = true,
    ALWAYS_CAST = true,
    EXTRA_SLOTS = true,
    EXTRA_PERK = true,
    FASTER_WANDS = true,
    EXTRA_MANA = true,
    TELEKINESIS = true,
    HEARTS_MORE_EXTRA_HP = true,
    MAP = true,
    ADVENTURER = true,
    HOMUNCULUS = true,
    LUKKI_MINION = true,
    EDIT_WANDS_EVERYWHERE = true,
    ATTRACT_ITEMS = true,
    EXTRA_SHOP_ITEM = true,
    GENOME_MORE_LOVE = true,
    GENOME_MORE_HATRED = true,
    NO_MORE_SHUFFLE = true,
    UNLIMITED_SPELLS = true,
    TRICK_BLOOD_MONEY = true,
    GOLD_IS_FOREVER = true,
    PEACE_WITH_GODS = true,
    GLOBAL_GORE = true,
    EXTRA_MONEY_TRICK_KILL = true,
    EXTRA_MONEY = true,
}

local global_perks = {
    NO_MORE_SHUFFLE = true,
    UNLIMITED_SPELLS = true,
    TRICK_BLOOD_MONEY = true,
    GOLD_IS_FOREVER = true,
    EXTRA_MONEY_TRICK_KILL = true,
    EXTRA_MONEY = true,
    PEACE_WITH_GODS = true,
    EXTRA_SHOP_ITEM = true,
    GENOME_MORE_LOVE = true,
    GENOME_MORE_HATRED = true,
    GLOBAL_GORE = true,
}

local function set_lukki(entity, peer_id)
    for _, child in ipairs(EntityGetAllChildren(entity) or {}) do
        for _, sprite in ipairs(EntityGetComponent(child, "SpriteComponent") or {}) do
            local img = ComponentGetValue(sprite, "image_file")
            local new
            if img == "data/entities/misc/perks/attack_foot/limb_a.png" then
                new = "mods/quant.ew/files/system/player/tmp/" .. peer_id .. "_limb_a.png"
            elseif img == "data/entities/misc/perks/attack_foot/limb_B.png" then
                new = "mods/quant.ew/files/system/player/tmp/" .. peer_id .. "_limb_b.png"
            elseif img == "data/entities/misc/perks/attack_foot/knee.png" then
                new = "mods/quant.ew/files/system/player/tmp/" .. peer_id .. "_knee.png"
            end
            if new ~= nil then
                ComponentSetValue(sprite, "image_file", new)
            end
        end
    end
end

function perk_fns.get_my_perks()
    lazyload()
    local perks = {}
    for i=1, #perk_list do
        local perk_flag_name = get_perk_picked_flag_name(perk_list[i].id)
        local perk_count = tonumber(GlobalsGetValue(perk_flag_name .. "_PICKUP_COUNT", "0"))
        if perk_count > 0 then
            perks[perk_list[i].id] = perk_count
            if perk_list[i].id == "ATTACK_FOOT" then
                set_lukki(ctx.my_player.entity, ctx.my_id)
            end
        end
    end
    return perks
end

local function spawn_perk(perk_info, auto_pickup_entity)
    local x, y = EntityGetTransform(ctx.my_player.entity)
    if x == nil then
        return
    end
    local perk_entity = perk_spawn(x, y - 8, perk_info.id)
    if auto_pickup_entity then
        perk_pickup(perk_entity, auto_pickup_entity, nil, true, false)
    end
    local icon = EntityCreateNew()
    EntityAddTag(icon, "perk_entity")
    EntityAddComponent2(icon, "UIIconComponent", {icon_sprite_file = perk_info.ui_icon, name = perk_info.ui_name, description = perk_info.ui_description})
    EntityAddChild(ctx.my_player.entity, icon)
end

local to_spawn = {}

local function give_one_perk(entity_who_picked, perk_info, count, allow_globals, peer_id)
    lazyload()

    if perk_info.ui_icon ~= nil then
        local icon = EntityCreateNew()
        EntityAddTag(icon, "perk_entity")
        EntityAddComponent2(icon, "UIIconComponent", {icon_sprite_file = perk_info.ui_icon, name = perk_info.ui_name, description = perk_info.ui_description})
        EntityAddChild(entity_who_picked, icon)
    end

    if not perks_to_ignore[perk_info.id] or (allow_globals and global_perks[perk_info.id]) then
        -- add game effect
        if perk_info.game_effect ~= nil then
            local game_effect_comp, ent = GetGameEffectLoadTo( entity_who_picked, perk_info.game_effect, true )
            if game_effect_comp ~= nil then
                ComponentSetValue( game_effect_comp, "frames", "-1" )
                EntityAddTag( ent, "perk_entity" )
            end
        end

        if perk_info.game_effect2 ~= nil then
            local game_effect_comp, ent = GetGameEffectLoadTo( entity_who_picked, perk_info.game_effect2, true )
            if game_effect_comp ~= nil then
                ComponentSetValue( game_effect_comp, "frames", "-1" )
                EntityAddTag( ent, "perk_entity" )
            end
        end

        if perk_info.func ~= nil then
            perk_info.func( 0, entity_who_picked, "", count )
            if perk_info.id == "ATTACK_FOOT" and peer_id ~= nil then
                set_lukki(entity_who_picked, peer_id)
            end
        end

        local no_remove = perk_info.do_not_remove or false

        -- particle effect only applied once
        if perk_info.particle_effect ~= nil and ( count <= 1 ) then
            local particle_id = EntityLoad( "data/entities/particles/perks/" .. perk_info.particle_effect .. ".xml" )

            if ( no_remove == false ) then
                EntityAddTag( particle_id, "perk_entity" )
            end

            EntityAddChild( entity_who_picked, particle_id )
        end
    end
end

local function deal_with_globals(perk_id, count)
    if global_perks[perk_id] then
        if to_spawn[perk_id] ~= nil then
            to_spawn[perk_id] = math.max(to_spawn[perk_id], count)
        else
            to_spawn[perk_id] = count
        end
    end
end

local wait_for_globals

function perk_fns.update_perks(perk_data, player_data)
    lazyload()
    local entity = player_data.entity
    local current_counts = util.get_ent_variable(entity, "ew_current_perks") or {}
    for perk_id, count in pairs(perk_data) do
        local current = (current_counts[perk_id] or 0)
        local diff = count - current
        if diff ~= 0 then
            local perk_info = get_perk_with_id(perk_list, perk_id)
            if perk_info == nil then
                print("Unknown perk id: "..perk_id)
                goto continue
            end
            if diff > 0 then
                print("Player " .. player_data.name .. " got perk " .. GameTextGetTranslatedOrNot(perk_info.ui_name))
                for i=current+1, count do
                    give_one_perk(entity, perk_info, i, false, player_data.peer_id)
                end
                deal_with_globals(perk_id, count)
            else
                wait_for_globals = GameGetFrameNum() + 60 * 60 * 20
                EntityKill(entity)
            end
        end
        ::continue::
    end

    util.set_ent_variable(entity, "ew_current_perks", perk_data)
end

function perk_fns.update_perks_for_entity(perk_data, entity, allow_perk)
    lazyload()
    local current_counts = util.get_ent_variable(entity, "ew_current_perks") or {}
    for perk_id, count in pairs(perk_data) do
        local current = (current_counts[perk_id] or 0)
        local diff = count - current
        if diff ~= 0 then
            local perk_info = get_perk_with_id(perk_list, perk_id)
            if perk_info == nil then
                print("Unknown perk id: "..perk_id)
                goto continue
            end
            if diff > 0 then
                if allow_perk(perk_info.id) then
                    for i=current+1, count do
                        give_one_perk(entity, perk_info, i, true)
                    end
                end
            end
        end
        ::continue::
    end

    -- This is NOT done here
    -- util.set_ent_variable(entity, "ew_current_perks", perk_data)
end

function perk_fns.on_world_update()
    if GameGetFrameNum() % 60 == 40
            and (wait_for_globals == nil or GameGetFrameNum() > wait_for_globals)
            and not EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
        wait_for_globals = nil
        for perk_id, num in pairs(to_spawn) do
            local n = perk_fns.get_my_perks()[perk_id] or 0
            if num > n then
                local perk_info = get_perk_with_id(perk_list, perk_id)
                for _ = 1, num - n do
                    spawn_perk(perk_info, true)
                end
            end
        end
        to_spawn = {}
    end
end

return perk_fns