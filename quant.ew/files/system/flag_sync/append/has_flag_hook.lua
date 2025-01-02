local old = HasFlagPersistent
local old_add_flag = AddFlagPersistent

function HasFlagPersistent(flag)
    if EwHasPersistentFlag ~= nil then
        return EwHasPersistentFlag(flag)
    end
    if CrossCall ~= nil then
        return CrossCall("ew_has_flag", flag)
    end
    print("the flag, " .. flag .. " is not being called in a synced way")
    return old(flag)
end

function AddFlagPersistent(flag)
    GameAddFlagRun("ew_pf_" .. flag)
    return old_add_flag(flag)
end
