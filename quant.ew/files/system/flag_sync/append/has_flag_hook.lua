function HasFlagPersistent(flag)
    if EwHasPersistentFlag ~= nil then
        return EwHasPersistentFlag(flag)
    end
    return CrossCall("ew_has_flag", flag)
end