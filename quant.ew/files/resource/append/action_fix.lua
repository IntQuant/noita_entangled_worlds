
-- loop backwards through perk_list so we can remove entries
for i=#actions,1,-1 do
    local action = actions[i]

    local func = action.action
    action.action = function(...)
        if reflecting then
            func(...)
            return
        end

        local oldSetRandomSeed = SetRandomSeed

        local shooter = EntityGetRootEntity(GetUpdatedEntityID())
        local x, y = EntityGetTransform(GetUpdatedEntityID())

        local seed = math.floor(x * y + GameGetFrameNum())

        if EntityHasTag(shooter, "ew_client") then
            seed = tonumber(GlobalsGetValue("ew_action_rng_"..EntityGetName(shooter), "0")) or 0
        else
            if GlobalsGetValue("ew_player_action_rng", "0") ~= "0" then
                seed = tonumber(GlobalsGetValue("ew_player_action_rng", "0"))
            else
                GlobalsSetValue("ew_player_action_rng", tostring(seed))
            end
        end

        SetRandomSeed = function()
            oldSetRandomSeed(seed, seed)
        end

        func(...)

        SetRandomSeed = oldSetRandomSeed
    end

end

local orig = GetUpdatedEntityID

function GetUpdatedEntityID()
    local ent = EntityGetRootEntity(orig())
    if EntityHasTag(ent, "ew_synced_entity") then
        ent = (EntityGetAllChildren(ent) or {0})[1]
    end
    return ent
end