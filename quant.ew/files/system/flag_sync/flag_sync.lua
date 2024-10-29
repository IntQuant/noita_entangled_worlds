local flag_present = {}

for _, flag in ipairs(util.string_split(ctx.proxy_opt.progress, ",")) do
    flag_present[flag] = true
end

local function has_flag(flag)
    return flag_present[flag] == true or GameHasFlagRun("ew_pf_"..flag)
end

function EwHasPersistentFlag(flag)
    return has_flag(flag)
end

np.CrossCallAdd("ew_has_flag", has_flag)

return {}
