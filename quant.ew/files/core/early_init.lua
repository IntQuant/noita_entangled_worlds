-- Early init stuff, called before main "mod" is loaded. Meaning we can append to data/scripts/init.lua
ModLuaFileAppend("data/scripts/init.lua", "mods/quant.ew/files/resource/append/no_default_death_handling.lua")

-- All files that use HasFlagPersistent.
local uses_persistent_flags = {
    "data/scripts/debug/persistent_flags_check.lua",
    "data/scripts/init.lua",
    "data/scripts/items/utility_box.lua",
    "data/scripts/items/generate_shop_item.lua",
    "data/scripts/items/make_random_card.lua",
    "data/scripts/gun/gun_actions.lua",
    "data/scripts/items/chest_random.lua",
    "data/scripts/buildings/teleroom.lua",
    "data/scripts/item_spawnlists.lua",
    "data/scripts/magic/moon_altar_init.lua",
    "data/scripts/magic/dark_moon_altar.lua",
    "data/scripts/magic/dark_moon_altar_init.lua",
    "data/scripts/magic/moon_altar.lua",
    "data/scripts/biome_modifiers.lua",
    "data/scripts/biomes/scale.lua",
    
    -- The tree shows your progress, should be fine to not sync it
    -- "data/scripts/biomes/mountain_tree.lua",
    
    -- The following also modify flags.
    -- "data/scripts/animals/boss_dragon_death.lua",
    -- "data/scripts/buildings/huussi.lua",
    -- "data/scripts/perks/perk.lua",
    -- "data/scripts/buildings/chest_light.lua",
    -- "data/scripts/buildings/chest_dark.lua",
    -- "data/scripts/magic/amulet.lua",
    -- "data/entities/animals/boss_alchemist/death.lua",
    -- "data/entities/animals/boss_pit/boss_pit_death.lua",
    -- "data/entities/animals/boss_limbs/boss_limbs_death.lua",
}

-- Override HasFlagPersistent in each.
for _, file in ipairs(uses_persistent_flags) do
    ModLuaFileAppend(file, "mods/quant.ew/files/system/flag_sync/append/has_flag_hook.lua")
end