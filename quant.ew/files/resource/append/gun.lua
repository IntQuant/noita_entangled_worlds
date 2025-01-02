local old_order_deck = order_deck

order_deck = function()
    local oldSetRandomSeed = SetRandomSeed
    SetRandomSeed = function()
        local shooter = EntityGetRootEntity(GetUpdatedEntityID())

        oldSetRandomSeed(GameGetFrameNum(), GameGetFrameNum())

        local seed = 0
        if EntityHasTag(shooter, "ew_client") then
            seed = tonumber(GlobalsGetValue("ew_shooter_rng_" .. EntityGetName(shooter), "0")) or 0
        elseif EntityHasTag(shooter, "player_unit") then
            seed = Random(10, 10000000)
            GlobalsSetValue("ew_player_rng", tostring(seed))
        end

        oldSetRandomSeed(seed, seed)
    end

    SetRandomSeed(0, 0)

    old_order_deck()

    SetRandomSeed = oldSetRandomSeed
end
