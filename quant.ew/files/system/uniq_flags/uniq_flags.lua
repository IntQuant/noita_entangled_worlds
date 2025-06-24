local net_handling = dofile_once("mods/quant.ew/files/core/net_handling.lua")

local module = {}

local function request_flag(flag)
    net.send_flags("0" .. flag)
end

function module.request_flag(flag)
    local current = coroutine.running()
    net_handling.pending_requests[flag] = current
    request_flag(flag)
    return coroutine.yield()
end

local function request_flag_slow(flag, ent)
    net.send_flags("1" .. ent .. " " .. flag)
end

function module.on_new_entity(arr)
    for _, ent in ipairs(arr) do
        if not EntityHasTag(ent, "ew_des") and EntityGetRootEntity(ent) == ent then
            local f = EntityGetFilename(ent)
            local seed = EntityGetFirstComponentIncludingDisabled(ent, "PositionSeedComponent")
            local x, y = EntityGetTransform(ent)
            local lx, ly = math.floor(x / 64), math.floor(y / 64)
            if EntityHasTag(ent, "ew_unique") then
                local flag = f .. ":" .. math.floor(x / 512) .. ":" .. math.floor(y / 512)
                ewext.notrack(ent)
                request_flag_slow(flag, ent)
            elseif
                (
                    f == "data/entities/props/physics_fungus.xml"
                    and (lx == -29 or lx == -28 or lx == -27)
                    and (ly == -20 or ly == -19)
                )
                or (f == "data/entities/props/physics_fungus_big.xml" and lx == -29 and ly == -20)
                or (f == "data/entities/props/physics_fungus_small.xml" and lx == -27 and ly == -19)
                or (f == "data/entities/items/pickup/evil_eye.xml" and lx == -39 and ly == -4)
            then
                local flag = f .. ":" .. lx .. ":" .. ly
                ewext.notrack(ent)
                request_flag_slow(flag, ent)
            elseif seed ~= nil then
                local flag = f .. ":" .. ComponentGetValue2(seed, "pos_x") .. ":" .. ComponentGetValue2(seed, "pos_y")
                ewext.notrack(ent)
                request_flag_slow(flag, ent)
            end
        end
    end
end

local function request_moon_flag_slow(x, y, dark)
    local flag = "ew_moon_spawn" .. ":" .. math.floor(x / 512) .. ":" .. math.floor(y / 512)
    local b = "0"
    if dark then
        b = "1"
    end
    net.send_flags("2" .. math.floor(x) .. " " .. math.floor(y) .. " " .. b .. " " .. flag)
end

util.add_cross_call("ew_moon_spawn", function(x, y, dark)
    request_moon_flag_slow(x, y, dark)
end)

return module
