local old = HasFlagPersistent

function HasFlagPersistent(flag)
    if EwHasPersistentFlag ~= nil then
        return EwHasPersistentFlag(flag)
    end
    if CrossCall ~= nil then
        return CrossCall("ew_has_flag", flag)
    else
        print("the flag, " .. flag .. " is not being called in a synced way")
        return old(flag)
    end
end