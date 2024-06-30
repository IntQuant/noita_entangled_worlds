dofile_once("data/scripts/perks/perk_list.lua")
local util = dofile_once("mods/quant.ew/files/src/util.lua")

local perk_fns = {}

-- Which perks we do not add to clients.
local perks_to_ignore = {
    GAMBLE = true, -- Tends to get readded, causing players to get a lot of random perks.
    -- Doesn't make sense to duplicate those to clients.
    PERKS_LOTTERY = true,
    REMOVE_FOG_OF_WAR = true,
    MEGA_BEAM_STONE = true,
    -- TODO: Needs extra work to work correctly
    -- NO_MORE_SHUFFLE = true,
    -- UNLIMITED_SPELLS = true,
}

function perk_fns.get_my_perks()
    local perks = {}
    for i=1, #perk_list do
        local perk_flag_name = get_perk_picked_flag_name(perk_list[i].id)
        local perk_count = tonumber(GlobalsGetValue(perk_flag_name .. "_PICKUP_COUNT", "0"))
        if perk_count > 0 then
            perks[perk_list[i].id] = perk_count
        end
    end
    return perks
end

local function give_one_perk(entity_who_picked, perk_info, count)
    if not perks_to_ignore[perk_info.id] then
        -- add game effect
        if perk_info.game_effect ~= nil then
            local game_effect_comp = GetGameEffectLoadTo( entity_who_picked, perk_info.game_effect, true )
            if game_effect_comp ~= nil then
                ComponentSetValue( game_effect_comp, "frames", "-1" )
            end
        end

        if perk_info.game_effect2 ~= nil then
            local game_effect_comp = GetGameEffectLoadTo( entity_who_picked, perk_info.game_effect2, true )
            if game_effect_comp ~= nil then
                ComponentSetValue( game_effect_comp, "frames", "-1" )
            end
        end

        if perk_info.func ~= nil then
            GamePrint(count)
            perk_info.func( 0, entity_who_picked, "", count )
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

function perk_fns.update_perks(perk_data, player_data)
    local entity = player_data.entity
    local current_counts = util.get_ent_variable(entity, "ew_current_perks") or {}
    for perk_id, count in pairs(perk_data) do 
        local current = (current_counts[perk_id] or 0)
        local diff = count - current
        -- TODO handle diff < 0?
        if diff ~= 0 then
            local perk_info = get_perk_with_id(perk_list, perk_id)
            if perk_info == nil then
                print("Unknown perk id: "..perk_id)
                goto continue
            end
            if diff > 0 then
                GamePrint("Player " .. player_data.name .. " got perk " .. GameTextGetTranslatedOrNot(perk_info.ui_name))
                for i=current+1, count do
                    give_one_perk(entity, perk_info, i)
                end
            end 
        end
        ::continue::
    end

    util.set_ent_variable(entity, "ew_current_perks", perk_data)
end

return perk_fns