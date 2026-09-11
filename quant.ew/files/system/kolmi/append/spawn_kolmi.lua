local old = item_pickup
function item_pickup(ent, who, name, run)
    if run then
        old(ent, who, name)
    else
        -- The fight starts on whichever peer owns Kolmi, which may not be this
        -- one, so a pickup here only gets reported. See kolmi.lua.
        CrossCall("ew_sampo_picked")
    end
end
