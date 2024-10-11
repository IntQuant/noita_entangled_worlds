local old = HasFlagPersistent

function HasFlagPersistent(flag)
    if EwHasPersistentFlag ~= nil then
        return EwHasPersistentFlag(flag)
    end
    if CrossCall ~= nil then
        return CrossCall("ew_has_flag", flag)
    else
        return old(flag)
    end
end