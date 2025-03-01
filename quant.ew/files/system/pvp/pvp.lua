local rpc = net.new_rpc_namespace()
local pvp = {}

local chunks_by_floor = {
    { { -3, 1, 3, 1 }, { 0, 0, 2, 0 } }, --1st biome
    { { -6, 3, 3, 4 }, { -7, 3, -7, 3 } }, --2nd biome
    { { -5, 6, 4, 8 } }, --3rd biome
    { { -4, 10, 2, 11 } }, --4th biome
    { { -5, 13, 3, 15 } }, --5th biome
    { { -6, 17, 4, 19 } }, --6th biome
    { { -6, 21, 4, 23 }, { -6, 24, 2, 24 } }, --7th biome
    { { 16, 1, 22, 5 } }, --sand cave
    { { -9, 0, -7, 0 }, { -9, 1, -5, 1 } }, --alchemist boss
    { { 17, 0, 21, 0 }, { 18, -1, 20, -1 } }, --temple boss
    { { -20, 25, -15, 28 } }, --ice biome
    { { -23, 1, -17, 5 } }, --surface robot biome
    { { 24, 4, 30, 8 } }, --wand mart
    { { 24, 16, 30, 20 } }, --robot biome
    { { -10, 15, -9, 18 }, { -8, 14, -7, 17 } }, --lukki lair
    { { -31, -13, -21, 10 } }, --clouds
    { { -6, 29, 4, 32 } }, --hell
    { { -6, -19, 4, -16 } }, --heaven
    { { 12, 14, 14, 17 }, { 13, 11, 14, 13 } }, --meat realm 1
    { { 27, 27, 30, 31 } }, --meat realm 2
    { { 18, 9, 20, 16 } }, --tower
}

local player_count = tonumber(GlobalsGetValue("ew_player_count", "1")) or 1

local my_num = tonumber(GlobalsGetValue("ew_num", "0")) or 0

local my_pw = tonumber(GlobalsGetValue("ew_pw", "0")) or 0

local floor = tonumber(GlobalsGetValue("ew_floor", "1")) or 1

local my_wins = tonumber(GlobalsGetValue("ew_wins", "0")) or 0

local player_died = {}

for _ = 1, #chunks_by_floor do
    table.insert(player_died, {})
end

--TODO give tiny platform so you dont fall in lava after tp

--TODO regenerate final hm

local hm_x = -677

local hm_ys = {
    1336,
    2872,
    4920,
    6456,
    8504,
    10552,
}

function rpc.recv_player_num(num, peer)
    if ctx.my_id == peer then
        my_num = num
        player_count = num
        my_pw = math.ceil(my_num / 2)
        if my_num % 2 == 0 then
            my_pw = -my_pw
        end
        hm_x = my_pw * BiomeMapGetSize() - 677
        GlobalsSetValue("ew_num", tostring(my_num))
        GlobalsSetValue("ew_pw", tostring(my_pw))
    end
end

rpc.opts_reliable()
function rpc.get_player_num()
    if ctx.is_host then
        rpc.recv_player_num(player_count, ctx.rpc_peer_id)
        player_count = player_count + 1
        GlobalsSetValue("ew_player_count", tostring(player_count))
    end
end

rpc.opts_everywhere()
function rpc.win(num)
    GamePrintImportant(ctx.rpc_player_data.name .. " wins, score: " .. tostring(num))
end

rpc.opts_everywhere()
function rpc.died(f)
    table.insert(player_died[f], ctx.rpc_peer_id)
end

local function float()
    local character_data = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "CharacterDataComponent")
    ComponentSetValue2(character_data, "mVelocity", 0, -20)
end

local function set_camera_pos(x, y)
    local cam = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "PlatformShooterPlayerComponent")
    if EntityHasTag(ctx.my_player.entity, "polymorphed_cessation") then
        return
    end
    if cam ~= nil and x ~= nil then
        ComponentSetValue2(cam, "mDesiredCameraPos", x, y)
    end
end

local function tp(x, y)
    set_camera_pos(x, y)
    EntitySetTransform(ctx.my_player.entity, x, y)
    async(function()
        wait(4)
        EntitySetTransform(ctx.my_player.entity, x, y)
        float()
    end)
end

local hm_y

function pvp.move_next_hm(died)
    hm_y = hm_ys[math.min(floor, #hm_ys)]
    tp(hm_x, hm_y)
    if died then
        rpc.died(floor)
    end
    floor = floor + 1
    GlobalsSetValue("ew_floor", tostring(floor))
end

function pvp.teleport_into_biome()
    local n = floor % #chunks_by_floor
    if n == 0 then
        n = #chunks_by_floor
    end
    local chunks_ranges = chunks_by_floor[n]
    local chunks = {}
    for _, chunk_range in ipairs(chunks_ranges) do
        local x1, y1, x2, y2 = chunk_range[1], chunk_range[2], chunk_range[3], chunk_range[4]
        for x = x1, x2 do
            for y = y1, y2 do
                table.insert(chunks, { x, y })
            end
        end
    end
    SetRandomSeed(tonumber(string.sub(ctx.my_id, 8, 12), 16), tonumber(string.sub(ctx.my_id, 12), 16))
    n = Random(1, #chunks)
    local my_chunk = chunks[n]
    local x = my_chunk[1] * 512 + 256
    local y = my_chunk[2] * 512 + 256
    tp(x, y)
    async(function()
        wait(8)
        local com = EntityCreateNew()
        EntityAddChild(ctx.my_player.entity, com)
        EntityAddComponent2(com, "GameEffectComponent", {
            effect = "TELEPORTATION",
            frames = 30,
            teleportation_probability = 1,
            teleportation_delay_min_frames = 60,
            teleportation_radius_min = 1,
            teleportation_radius_min = 256,
        })
        float()
        wait(4)
        x, y = EntityGetTransform(ctx.my_player.entity)
        LoadPixelScene("mods/quant.ew/files/system/pvp/tp.png", "", x - 6, y - 13, "", true, true) --ff5e9ab5
    end)
    hm_y = nil
end

local first = true

function pvp.on_world_update()
    if first then
        if ctx.is_host then
            pvp.teleport_into_biome()
            first = false
        elseif ctx.players[ctx.host_id] ~= nil then
            rpc.get_player_num()
            pvp.teleport_into_biome()
            first = false
        end
    end
    local _, y = EntityGetTransform(ctx.my_player.entity)
    if hm_y ~= nil and math.floor(hm_y / 512) ~= math.floor(y / 512) then
        pvp.teleport_into_biome()
    end
    if
        player_count ~= 1
        and GameGetFrameNum() % 60 == 32
        and #player_died[floor] == #ctx.players - 1
        and not table.contains(player_died[floor], ctx.my_id)
    then
        pvp.move_next_hm(false)
        my_wins = my_wins + 1
        GlobalsSetValue("ew_wins", tostring(my_wins))
        rpc.win(my_wins)
    end
end

function pvp.on_new_entity(ent)
    if
        EntityGetFirstComponentIncludingDisabled(ent, "TeleportComponent") ~= nil
        and string.sub(EntityGetFilename(ent), 1, 24) == "data/entities/buildings/"
    then
        EntityKill(ent)
    end
end

return pvp
