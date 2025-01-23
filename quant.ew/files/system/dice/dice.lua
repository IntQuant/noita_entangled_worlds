util.prepend(
    "data/scripts/items/greed_die_status.lua",
    "SetRandomSeed( GameGetFrameNum(), pos_x + pos_y + entity_id )",
    'local function get_num() if get_variable_storage_component(entity_id, "ew_rng") then return ComponentGetValue2(get_variable_storage_component(entity_id, "ew_rng"), "value_int") else return 0 end end SetRandomSeed(get_num(), 0)'
)
util.prepend(
    "data/scripts/items/die_status.lua",
    "SetRandomSeed( GameGetFrameNum(), pos_x + pos_y + entity_id )",
    'local function get_num() if get_variable_storage_component(entity_id, "ew_rng") then return ComponentGetValue2(get_variable_storage_component(entity_id, "ew_rng"), "value_int") else return 0 end end SetRandomSeed(get_num(), 0)'
)
util.prepend(
    "data/scripts/items/die_status.lua",
    'bullet_circle( "fungus", 8, 300 )',
    'if CrossCall("ew_do_i_own", entity_id) then bullet_circle("fungus", 8, 300) end'
)

return {}
