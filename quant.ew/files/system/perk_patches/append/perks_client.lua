if not ctx.proxy_opts.randomize_perks then
    return
end

local function hide_perk(perk_id)
    local perk_data = get_perk_with_id(perk_list, perk_id)
    perk_data.not_in_default_perk_pool = true
end

hide_perk("TELEKINESIS")