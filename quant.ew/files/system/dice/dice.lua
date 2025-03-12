local s = "local function get_num()\n"
    .. "local var\n"
    .. 'for _, v in ipairs(EntityGetComponentIncludingDisabled(entity_id, "VariableStorageComponent") or {}) do\n'
    .. 'if ComponentGetValue2(v, "name") == "ew_rng" then\n'
    .. "var = v\n"
    .. "end\n"
    .. "end\n"
    .. "if var ~= nil then\n"
    .. 'return ComponentGetValue2(var, "value_int")\n'
    .. "else\n"
    .. "return 0\n"
    .. "end\n"
    .. "end\n"
    .. "SetRandomSeed(get_num(), 0)\n"
    .. 'local util = dofile_once("mods/quant.ew/files/resource/util_min.lua")\n'
    .. "local shoot = shoot_projectile\n"
    .. "function shoot_projectile(...)\n"
    .. "if util.do_i_own(entity_id) then\n"
    .. "shoot(...)\n"
    .. "end\n"
    .. "end"
util.prepend(
    "data/scripts/items/greed_die_status.lua",
    "SetRandomSeed( GameGetFrameNum(), pos_x + pos_y + entity_id )",
    s
)
util.prepend("data/scripts/items/die_status.lua", "SetRandomSeed( GameGetFrameNum(), pos_x + pos_y + entity_id )", s)

s = "GetUpdatedEntityID()\n"
    .. 'local util = dofile_once("mods/quant.ew/files/resource/util_min.lua")\n'
    .. "if not util.do_i_own(entity_id) then\n"
    .. "return\n"
    .. "end"
local lst = {
    "data/scripts/items/broken_wand_spells.lua",
    "data/scripts/items/die_roll.lua",
    "data/scripts/buildings/sun/spot_2.lua",
    "data/scripts/items/broken_wand_throw.lua",
}
for _, f in ipairs(lst) do
    util.prepend(f, "GetUpdatedEntityID()", s)
end
return {}
