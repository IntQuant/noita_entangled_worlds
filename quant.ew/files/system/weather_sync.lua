local net = dofile_once("mods/quant.ew/files/core/net.lua")

local rpc = net.new_rpc_namespace()

local module = {}

-- Not actually sure what most of these do, but they clearly are weather-related, so we'll sync them anyway.
local fields = {
    "time", "time_total",
    "day_count",
    "rain", "rain_target",
    "fog", "fog_target",
    "wind", "wind_speed",
    -- "wind_speed_sin_t", "wind_speed_sin", "clouds_01_target", "clouds_02_target", "gradient_sky_alpha_target",
    -- "sky_sunset_alpha_target",
}

function module.on_world_update_host()
    if GameGetFrameNum() % 120 ~= 0 then
        return
    end
    local ws = GameGetWorldStateEntity()
    local wsc = EntityGetFirstComponentIncludingDisabled(ws, "WorldStateComponent")
    local weather_pattern = {}
    for _, field in ipairs(fields) do
        table.insert(weather_pattern, ComponentGetValue2(wsc, field))
    end
    rpc.apply_weather(weather_pattern)
end

function rpc.apply_weather(weather_pattern)
    local ws = GameGetWorldStateEntity()
    local wsc = EntityGetFirstComponentIncludingDisabled(ws, "WorldStateComponent")

    for i, field in ipairs(fields) do
        ComponentSetValue2(wsc, field, weather_pattern[i])
    end
end

return module