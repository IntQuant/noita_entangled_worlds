local function hide_perk(perk_id)
    local perk_data = get_perk_with_id(perk_list, perk_id)
    perk_data.not_in_default_perk_pool = true
end

hide_perk("ABILITY_ACTIONS_MATERIALIZED")
hide_perk("TELEKINESIS")
hide_perk("INVISIBILITY")
hide_perk("CORDYCEPS")
hide_perk("HOMUNCULUS")
hide_perk("ATTACK_FOOT")
hide_perk("ANGRY_GHOST")