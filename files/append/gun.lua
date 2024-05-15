local old_order_deck = order_deck 

order_deck = function()
    local oldSetRandomSeed = SetRandomSeed
    SetRandomSeed = function()

        local shooter = EntityGetRootEntity(GetUpdatedEntityID())

        --GamePrint(EntityGetName(shooter))

        oldSetRandomSeed(GameGetFrameNum(), GameGetFrameNum())

        local seed = 0
        if(EntityHasTag(shooter, "ew_client"))then
            -- GamePrint("2: ew_shooter_rng_"..EntityGetName(shooter))
            -- GamePrint(GlobalsGetValue("ew_shooter_rng_"..EntityGetName(shooter), "0"))
            seed = tonumber(GlobalsGetValue("ew_shooter_rng_"..EntityGetName(shooter), "0")) or 0
        elseif(EntityHasTag(shooter, "player_unit"))then
            seed = Random(10, 10000000)
            GlobalsSetValue("ew_player_rng", tostring(seed))
            -- GamePrint(tostring(seed))
        end

        oldSetRandomSeed(seed, seed)
    end

    SetRandomSeed()

    old_order_deck()

    SetRandomSeed = oldSetRandomSeed
end
