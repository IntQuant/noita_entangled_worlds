local s = 'local sx, sy = CrossCall("ew_per_peer_seed")\n'
    .. "rnd = SetRandomSeed\n"
    .. "local function SetRandomSeed(x, y)\n"
    .. "return rnd(x + sx, y + sy)\n"
    .. "end\n"
    .. "SetRandomSeed"
local f = {
    "data/scripts/gun/procedural/wand_petri.lua",
    "data/scripts/gun/procedural/chargegun.lua",
    "data/scripts/gun/procedural/digger_01_setup.lua",
    "data/scripts/gun/procedural/elite_machinegun.lua",
    "data/scripts/gun/procedural/elite_pistol.lua",
    "data/scripts/gun/procedural/elite_shotgun.lua",
    "data/scripts/gun/procedural/general_gun.lua",
    "data/scripts/gun/procedural/gun_procedural.lua",
    "data/scripts/gun/procedural/gun_procedural_better.lua",
    "data/scripts/gun/procedural/handgun.lua",
    "data/scripts/gun/procedural/level_1_wand.lua",
    "data/scripts/gun/procedural/machinegun.lua",
    "data/scripts/gun/procedural/nukelauncher.lua",
    "data/scripts/gun/procedural/rocketlauncher.lua",
    "data/scripts/gun/procedural/shotgun.lua",
    "data/scripts/gun/procedural/starting_bomb_wand.lua",
    "data/scripts/gun/procedural/starting_bomb_wand_daily.lua",
    "data/scripts/gun/procedural/starting_wand.lua",
    "data/scripts/gun/procedural/starting_wand_daily.lua",
    "data/scripts/gun/procedural/submachinegun.lua",
    "data/scripts/gun/procedural/wand_daily.lua",
    "data/scripts/items/potion_starting.lua",
}
for _, p in ipairs(f) do
    util.prepend(p, "SetRandomSeed", s)
end
return {}
