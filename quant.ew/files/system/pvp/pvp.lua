local rpc = net.new_rpc_namespace()
local pvp = {}

local chunks_by_floor = {
    { { -3, 1, 3, 1 }, { 0, 0, 2, 0 } }, --(Collapsed) Mines
    { { -6, 3, 3, 4 }, { -7, 3, -7, 3 } }, --Coal Pits
    { { -5, 6, 4, 8 } }, --Snowy Depths
    { { -4, 10, 2, 11 } }, --Hiisi Base
    { { -5, 13, 3, 15 } }, --Underground Jungle
    { { -6, 17, 4, 19 } }, --Vault
    { { -6, 21, 4, 23 }, { -6, 24, 2, 24 } }, --Temple of the Art
    { { -29, 1, -27, 0 } }, --Island
    { { 16, 1, 22, 5 }, { 17, 0, 21, 0 }, { 18, -1, 20, -1 } }, --Sand Cave/Pyramid
    { { -9, 0, -7, 0 }, { -9, 1, -5, 1 } }, --Ancient Laboratory
    { { -31, -13, -21, 10 } }, --Cloudscape
    { { -20, 25, -15, 28 } }, --Snow Chasm
    { { -23, 1, -17, 5 } }, --Frozen Vault
    { { 24, 4, 30, 8 } }, --Overgrown Caverns
    { { 18, 25, 22, 30 } }, --Wizards Den
    { { -10, 15, -9, 18 }, { -8, 14, -7, 17 } }, --Lukki Lair
    { { 24, 16, 30, 20 } }, --Powerplant
    { { -6, 29, 4, 32 } }, --The Work(below)
    { { 27, 27, 30, 31 } }, --Meat Realm (Tiny)
    { { -6, -19, 4, -16 } }, --The Work(above)
    { { 12, 14, 14, 17 }, { 13, 11, 14, 13 } }, --Meat Realm (Heart)
    { { 18, 9, 20, 16 } }, --The Tower
}

local names_by_floor = {
    "(Collapsed) Mines",
    "Coal Pits",
    "Snowy Depths",
    "Underground Jungle",
    "Vault",
    "Temple of The Art",
    "Island",
    "Sand Cave/Pyramid",
    "Ancient Laboratory",
    "Cloudscape",
    "Frozen Vault",
    "Overgrown Caverns",
    "Wizards Den",
    "Lukki Lair",
    "Powerplant",
    "The Work (Below)",
    "Meat Realm (Tiny)",
    "The Work (Above)",
    "Meat Realm (Heart)",
    "The Tower",
}

if ModIsEnabled("Apotheosis") then
    chunks_by_floor = {
        { { -3, 1, 3, 1 }, { 0, 0, 2, 0 } }, --(Collapsed) Mines
        { { -6, 3, 3, 4 }, { -7, 3, -7, 3 } }, --Coal Pits
        { { -5, 6, 4, 8 } }, --Snowy Depths
        { { -4, 10, 2, 11 } }, --Hiisi Base
        { { -5, 13, 3, 15 } }, --Underground Jungle
        { { -6, 17, 4, 19 } }, --Vault
        { { -6, 21, 4, 23 }, { -6, 24, 2, 24 } }, --Temple of the Art
        { { -29, 1, -27, 0 } }, --Island
        { { 16, 1, 22, 5 }, { 17, 0, 21, 0 }, { 18, -1, 20, -1 } }, --Sand Cave/Pyramid
        { { -9, 0, -7, 0 }, { -9, 1, -5, 1 } }, --Ancient Laboratory
        { { -31, -13, -21, 10 } }, --Cloudscape
        { { -20, 25, -15, 28 } }, --Snow Chasm
        { { -23, 1, -17, 5 } }, --Frozen Vault
        { { 24, 4, 30, 8 } }, --Overgrown Caverns
        { { 18, 25, 22, 30 } }, --Wizards Den
        { { -10, 15, -9, 18 }, { -8, 14, -7, 17 } }, --Lukki Lair
        { { 24, 16, 30, 20 } }, --Powerplant
        { { -6, 29, 4, 32 } }, --The Work(below)
        { { -6, -19, 4, -16 } }, --The Work(above)
        { { 12, 14, 14, 17 }, { 13, 11, 14, 13 } }, --Meat Realm (Heart)
        { { 27, 27, 30, 31 } }, --Meat Realm (Tiny)
        { { 18, 9, 20, 16 } }, --The Tower
    }
    names_by_floor = {
        "Mines",
        "Coal Pits",
        "Snowy Depths",
        "Fungal Caverns",
        "Ant Nest",
        "Core Mines",
        "Hiisi Base",
        "Sand Cave/Pyramid",
        "Underground Jungle",
        "Ancient Laboratory",
        "Vault",
        "Lukki Lair",
        "The Work (Below)",
        "Frozen Vault",
        "Tower",
        "Island",
        "Overgrown Caverns",
        "Cloudscape",
        "Snowy Chasm",
        "Sunken Caverns",
        "Sinkhole",
        "Powerplant",
        "Virulent Caverns/Contaminated Outpost",
        "Sinkhole (Temple)",
        "Temple of the Art",
        "Meat Realm (Tiny)",
        "Wizards Den",
        "Meat Realm (Heart)",
        "The Work (Above)",
        "The Work (Below Below)",
        "Temple of Sacrilegious Remains",
    }
end

local needs_ase = {
    "Wizards Den",
    "Magical Temple",
    "Ant Nest",
    "Temple of Sacrilegious Remains",
}

local player_count = 1

local my_num = 0

local my_pw = 1

local floor = 1

local my_wins = 0

local player_died = {}

local players_by_floor = {}

local hm_x = -677

local temp_ase

pvp.last_damage = nil

local hm_ys = {
    1336,
    2872,
    4920,
    6456,
    8504,
    10552,
}

function rpc.recv_player_num(num, peer)
    player_count = num + 1
    if ctx.my_id == peer then
        my_num = num
        my_pw = math.ceil((my_num + 1) / 2)
        if my_num % 2 == 1 then
            my_pw = -my_pw
        end
        hm_x = my_pw * BiomeMapGetSize() * 512 - 677
        GlobalsSetValue("ew_num", tostring(my_num))
        GlobalsSetValue("ew_pw", tostring(my_pw))
    end
    GlobalsSetValue("ew_player_count", tostring(player_count))
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
    GamePrint(ctx.rpc_player_data.name .. " wins, score: " .. tostring(num))
    GamePrint("next biome: " .. names_by_floor[floor])
end

rpc.opts_everywhere()
function rpc.died(f)
    if player_died[f] == nil then
        player_died[f] = {}
    end
    table.insert(player_died[f], ctx.rpc_peer_id)
end

rpc.opts_everywhere()
rpc.opts_reliable()
function rpc.remove_floor(f)
    if players_by_floor[f] == nil then
        players_by_floor[f] = {}
    end
    players_by_floor[f][ctx.rpc_peer_id] = nil
end

rpc.opts_everywhere()
rpc.opts_reliable()
function rpc.add_floor(f)
    if players_by_floor[f] == nil then
        players_by_floor[f] = {}
    end
    players_by_floor[f][ctx.rpc_peer_id] = true
end

local function float()
    local character_data = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "CharacterDataComponent")
    ComponentSetValue2(character_data, "mVelocity", 0, -40)
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
    ctx.stop_cam = true
    set_camera_pos(x, y)
    EntitySetTransform(ctx.my_player.entity, x, y)
    async(function()
        wait(4)
        EntitySetTransform(ctx.my_player.entity, x, y)
        float()
    end)
end

function rpc.give_gold(peer, gold)
    if peer == ctx.my_id then
        local wallet = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "WalletComponent")
        if wallet ~= nil then
            local my_gold = ComponentGetValue2(wallet, "money")
            ComponentSetValue2(wallet, "money", my_gold + gold)
            GamePrint(
                "gained " .. tostring(math.ceil(gold)) .. " gold from killing: " .. ctx.players[ctx.rpc_peer_id].name
            )
        end
    end
end

local hm_y

dofile_once("data/scripts/perks/perk.lua")

function pvp.move_next_hm(died)
    hm_y = hm_ys[math.min(floor, #hm_ys)]
    tp(hm_x, hm_y)
    if floor > #hm_ys then
        local x, y = -480, 10564
        EntityLoad("data/entities/items/pickup/heart_fullhp_temple.xml", x - 16, y)
        EntityLoad("data/entities/items/pickup/spell_refresh.xml", x + 16, y)
        x, y = 0, 10625
        perk_spawn_many(x, y)
    end
    if died then
        rpc.died(floor)
        if pvp.last_damage ~= nil then
            local wallet = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "WalletComponent")
            if wallet ~= nil then
                local gold = ComponentGetValue2(wallet, "money")
                local rt = ctx.proxy_opt.pvp_kill_steal / 100
                rpc.give_gold(pvp.last_damage, gold * rt)
                ComponentSetValue2(wallet, "money", gold * (1 - rt))
                GamePrint(
                    "lost "
                        .. tostring(math.ceil(gold * rt))
                        .. " gold from dying to: "
                        .. ctx.players[pvp.last_damage].name
                )
            end
        end
    end
    rpc.remove_floor(floor)
    floor = floor + 1
    pvp.last_damage = nil
    GlobalsSetValue("ew_floor", tostring(floor))
    if table.contains(needs_ase, names_by_floor[floor]) then
        local has_ase = false
        for _, ent in ipairs(EntityGetAllChildren(ctx.my_player.entity)) do
            local com = EntityGetFirstComponentIncludingDisabled(ent, "GameEffectComponent")
            if com ~= nil and ComponentGetValue2(com, "effect") == "REMOVE_FOG_OF_WAR" then
                has_ase = true
                break
            end
        end
        if not has_ase then
            temp_ase = EntityCreateNew()
            EntityAddChild(ctx.my_player.entity, temp_ase)
            EntityAddComponent2(temp_ase, "GameEffectComponent", {
                effect = "REMOVE_FOG_OF_WAR",
                frames = -1,
            })
        end
    elseif temp_ase ~= nil then
        if EntityGetIsAlive(temp_ase) then
            EntityKill(temp_ase)
        else
            for _, ent in ipairs(EntityGetAllChildren(ctx.my_player.entity)) do
                local com = EntityGetFirstComponentIncludingDisabled(ent, "GameEffectComponent")
                if
                    com ~= nil
                    and ComponentGetValue2(com, "effect") == "REMOVE_FOG_OF_WAR"
                    and not EntityHasTag(ent, "perk_entity")
                then
                    EntityKill(ent)
                    break
                end
            end
        end
        temp_ase = nil
    end
end

function pvp.teleport_into_biome()
    rpc.add_floor(floor)
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
    local com = EntityCreateNew()
    EntityAddChild(ctx.my_player.entity, com)
    EntityAddComponent2(com, "GameEffectComponent", {
        effect = "PROTECTION_ALL",
        frames = 300,
    })
    async(function()
        wait(8)
        com = EntityCreateNew()
        EntityAddChild(ctx.my_player.entity, com)
        EntityAddTag(com, "perk_entity")
        EntityAddComponent2(com, "GameEffectComponent", {
            effect = "TELEPORTATION",
            frames = 4,
            teleportation_probability = 1,
            teleportation_delay_min_frames = 8,
            teleportation_radius_min = 1,
            teleportation_radius_min = 256,
        })
        float()
        wait(16)
        x, y = EntityGetTransform(ctx.my_player.entity)
        LoadPixelScene("mods/quant.ew/files/system/pvp/tp.png", "", x - 6, y - 6, "", true, true)
    end)
    hm_y = nil
end

local first = true

function pvp.on_world_update()
    if first then
        if not ctx.is_host and ctx.players[ctx.host_id] == nil then
            return
        end
        first = false
        player_count = tonumber(GlobalsGetValue("ew_player_count", "-1")) or 1
        my_num = tonumber(GlobalsGetValue("ew_num", "-1")) or 0
        my_pw = tonumber(GlobalsGetValue("ew_pw", "-1")) or 1
        floor = tonumber(GlobalsGetValue("ew_floor", "1")) or 1
        my_wins = tonumber(GlobalsGetValue("ew_wins", "0")) or 0
        if my_num == -1 then
            if ctx.is_host then
                my_num = 0
                my_pw = 1
                player_count = 1
                hm_x = BiomeMapGetSize() * 512 - 677
            else
                rpc.get_player_num()
            end
            pvp.teleport_into_biome()
        end
    end
    local _, y = EntityGetTransform(ctx.my_player.entity)
    if hm_y ~= nil and math.floor(hm_y / 512) ~= math.floor(y / 512) then
        local has_alive = false
        for _, _ in pairs(players_by_floor[floor - 1] or {}) do
            has_alive = true
            break
        end
        if ctx.proxy_opt.wait_on_players and has_alive then
            floor = floor - 1
            pvp.move_next_hm(false)
        else
            pvp.teleport_into_biome()
        end
    end
    if player_died[floor] == nil then
        player_died[floor] = {}
    end
    local dead = 0
    for _, _ in pairs(player_died[floor]) do
        dead = dead + 1
    end
    if
        player_count ~= 1
        and GameGetFrameNum() % 60 == 32
        and #player_died[floor] == player_count - 1
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
