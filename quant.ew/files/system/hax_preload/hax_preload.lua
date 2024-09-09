local function preload(x, y)
    local chunkloader = util.load_ephemerial("mods/quant.ew/files/resource/entities/chunk_loader.xml", x, y)
    async(function ()
        wait(3)
        local cx, cy = GameGetCameraPos()
        GameSetCameraPos(x, y)
        GameSetCameraPos(cx, cy)
        EntityKill(chunkloader)
    end)
end

local module = {}

local first_update = true

function module.on_world_update_host()
    if first_update then
        async(function ()
            preload(-300, 1400)
            wait(1)
            preload(-300, 2900)
            wait(1)
            preload(-300, 5000)
            wait(1)
            preload(-300, 6500)
            wait(1)
            preload(-300, 8550)
            wait(1)
            preload(-300, 10600)
            wait(1)
            preload(2200, 13150)
        end)
        first_update = false
    end
end

return module