local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local util = dofile_once("mods/quant.ew/files/src/util.lua")

if not ctx.is_host then
    -- Damage handling shouldn't run on clients.
    util.replace_text_in("data/entities/animals/boss_sky/boss_sky_damage.xml", 'script_source_file="data/entities/animals/boss_sky/boss_sky_damage.lua"', "")
end

return {}