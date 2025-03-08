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
    { { -29, -1, -27, 0 } }, --Island
    { { 16, 1, 22, 5 }, { 17, 0, 21, 0 }, { 18, -1, 20, -1 } }, --Sand Cave/Pyramid
    { { -9, 0, -7, 0 }, { -9, 1, -5, 1 } }, --Ancient Laboratory
    { { -31, -13, -21, -10 } }, --Cloudscape
    { { -20, 25, -15, 28 } }, --Snowy Chasm
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
    "Hiisi Base",
    "Underground Jungle",
    "Vault",
    "Temple of The Art",
    "Island",
    "Sand Cave/Pyramid",
    "Ancient Laboratory",
    "Cloudscape",
    "Snowy Chasm",
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
        { { 5, 11, 8, 12 } }, --Ant Nest
        { { 11, 11, 14, 17 } }, --Core Mines
        { { -4, 10, 2, 11 } }, --Hiisi Base
        { { 16, 1, 22, 5 }, { 17, 0, 21, 0 }, { 18, -1, 20, -1 } }, --Sand Cave/Pyramid
        { { -5, 13, 3, 15 } }, --Underground Jungle
        { { -9, 0, -7, 0 }, { -9, 1, -5, 1 } }, --Ancient Laboratory
        { { -6, 17, 4, 19 } }, --Vault
        { { -10, 15, -9, 18 }, { -8, 14, -7, 17 } }, --Lukki Lair
        { { -6, 29, 4, 32 } }, --The Work(below)
        { { -23, 1, -17, 5 } }, --Frozen Vault
        { { 18, 9, 20, 16 } }, --The Tower
        { { -29, -1, -27, 0 } }, --Island
        { { 24, 4, 30, 8 } }, --Overgrown Caverns
        { { -31, -13, -21, -10 } }, --Cloudscape
        { { -20, 25, -15, 28 } }, --Snowy Chasm
        { { -41, 15, -33, 18 } }, --Sunken Caverns
        { { 38, 2, 40, 4 } }, --Sinkhole
        { { 24, 16, 30, 20 } }, --Powerplant
        { { 34, 23, 38, 32 } }, --Virulent Caverns/Contaminated Outpost
        { { 34, 6, 35, 7 }, { 35, 4, 36, 5 } }, --Sinkhole(temple)
        { { -6, 21, 4, 23 }, { -6, 24, 2, 24 } }, --Temple of the Art
        { { 27, 27, 30, 31 } }, --Meat Realm (Tiny)
        { { 18, 25, 22, 30 } }, --Wizards Den
        { { 12, 18, 14, 20 } }, --Meat Realm (Heart)
        { { -6, -19, 4, -16 } }, --The Work(above)
        { { -6, 77, 4, 81 } }, --The Work(below below)
        { { 39, 8, 43, 17 }, { 44, 13, 47, 13 } }, --Temple of Sacrilegious Remains
    }
    names_by_floor = {
        "(Collapsed) Mines",
        "Coal Pits",
        "Snowy Depths",
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

pvp.floor = 1

local my_wins = 0

local player_died = {}

pvp.players_by_floor = {}

local hm_x = 0

local hm_y

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

local wins = {}

local boards = {}

local nickname = dofile_once("mods/quant.ew/files/system/nickname.lua")

local function create_board()
    for _, ent in ipairs(boards) do
        EntityKill(ent)
    end
    boards = {}
    if hm_y == nil then
        return
    end
    local sorted = {}
    local k = 0
    for p, n in pairs(wins) do
        k = k + 1
        if k > 16 then
            break
        end
        table.insert(sorted, { p, n })
    end
    table.sort(sorted, function(a, b)
        return a[2] > b[2]
    end)
    for i, v in ipairs(sorted) do
        local p = v[1]
        local n = v[2]
        local ent = EntityCreateNew()
        local x = hm_x - 556 + 18 * math.floor((i - 1) / 4)
        if i > 8 then
            x = x + 100
        end
        local y = hm_y - 26 + ((i - 1) % 4) * 18
        EntitySetTransform(ent, x, y)
        nickname.add_label(ent, tostring(n), "data/fonts/font_pixel_white.xml", 0.75, 0.5, -12, -6)
        table.insert(boards, ent)
        EntityAddComponent2(
            ent,
            "SpriteComponent",
            { image_file = "mods/quant.ew/files/system/player/tmp/" .. p .. "_icon.png", alpha = 0.5 }
        )
    end
end

function rpc.recv_player_num(num, peer)
    player_count = num + 1
    if ctx.my_id == peer then
        my_num = num
        my_pw = math.ceil((my_num + 1) / 2)
        if my_num % 2 == 1 then
            my_pw = -my_pw
        end
        hm_x = my_pw * BiomeMapGetSize() * 512
        GlobalsSetValue("ew_num", tostring(my_num))
        GlobalsSetValue("ew_pw", tostring(my_pw))
    end
    GlobalsSetValue("ew_player_count", tostring(player_count))
    if hm_y == nil then
        rpc.add_floor(pvp.floor, true)
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

local tmr

rpc.opts_everywhere()
function rpc.win(num)
    GamePrint(ctx.rpc_player_data.name .. " wins, score: " .. tostring(num))
    local n = pvp.floor % #names_by_floor
    if n == 0 then
        n = #names_by_floor
    end
    GamePrint("next biome: " .. names_by_floor[n])
    wins[ctx.rpc_peer_id] = num
    create_board()
    if ctx.my_id == ctx.rpc_peer_id and ctx.proxy_opt.chest_on_win then
        local ent = EntityLoad("data/entities/items/pickup/chest_random.xml", hm_x - 600, hm_y + 40)
        ewext.notrack(ent)
    end
    if ctx.is_host then
        tmr = nil
    end
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
    if pvp.players_by_floor[f] == nil then
        pvp.players_by_floor[f] = {}
    end
    pvp.players_by_floor[f][ctx.rpc_peer_id] = nil
end

rpc.opts_everywhere()
rpc.opts_reliable()
function rpc.add_floor(f, ping, msg)
    if pvp.players_by_floor[f] == nil then
        pvp.players_by_floor[f] = {}
    end
    pvp.players_by_floor[f][ctx.rpc_peer_id] = true
    if msg then
        local n = pvp.floor % #names_by_floor
        if n == 0 then
            n = #names_by_floor
        end
        GamePrint(ctx.rpc_player_data.name .. " has entered: " .. names_by_floor[n])
    end
    if ping and ctx.rpc_peer_id ~= ctx.my_id and hm_y == nil then
        rpc.add_floor(pvp.floor, false, false)
    end
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
            if math.ceil(gold) ~= 0 then
                GamePrint(
                    "gained "
                        .. tostring(math.ceil(gold))
                        .. " gold from killing: "
                        .. ctx.players[ctx.rpc_peer_id].name
                )
            end
        end
    end
end

dofile_once("data/scripts/items/generate_shop_item.lua")

local has_did = false

local function spawn_items(x, y)
    local n = pvp.floor % #chunks_by_floor
    if n == 0 then
        n = #chunks_by_floor
    end
    local dx = pvp.floor
    local dy = chunks_by_floor[n][1][1] - my_wins
    SetRandomSeed(x + dx, y + dy)
    local count = tonumber(GlobalsGetValue("TEMPLE_SHOP_ITEM_COUNT", "5"))
    local width = 132
    local item_width = width / count
    local sale_item_i = Random(1, count)
    for _, ent in ipairs(EntityGetWithTag("wand") or {}) do
        local cost = EntityGetFirstComponentIncludingDisabled(ent, "ItemCostComponent")
        if cost ~= nil and ComponentGetValue2(cost, "cost") > 0 then
            EntityKill(ent)
        end
    end
    for _, ent in ipairs(EntityGetWithTag("card_action") or {}) do
        local cost = EntityGetFirstComponentIncludingDisabled(ent, "ItemCostComponent")
        if cost ~= nil and ComponentGetValue2(cost, "cost") > 0 then
            EntityKill(ent)
        end
    end
    --if Random(0, 100) <= 50 then
    for i = 1, count do
        if i == sale_item_i then
            local ent = generate_shop_item(x + (i - 1) * item_width + dx, 13156 + dy, true, nil, true)
            EntitySetTransform(ent, x + (i - 1) * item_width, y)
        else
            local ent = generate_shop_item(x + (i - 1) * item_width + dx, 13156 + dy, false, nil, true)
            EntitySetTransform(ent, x + (i - 1) * item_width, y)
        end
        local ent = generate_shop_item(x + (i - 1) * item_width + dx, 13156 - 30 + dy, false, nil, true)
        EntitySetTransform(ent, x + (i - 1) * item_width, y - 30)
        if not has_did then
            LoadPixelScene(
                "data/biome_impl/temple/shop_second_row.png",
                "data/biome_impl/temple/shop_second_row_visual.png",
                x + (i - 1) * item_width - 8,
                y - 22,
                "",
                true
            )
        end
    end
    has_did = true
    --[[else
        for i = 1, count do
            if i == sale_item_i then
                local ent = generate_shop_wand(x + (i - 1) * item_width + dx, 13156 + dy, true)
                EntitySetTransform(ent, x + (i - 1) * item_width, y)
            else
                local ent = generate_shop_wand(x + (i - 1) * item_width + dx, 13156 + dy, false)
                EntitySetTransform(ent, x + (i - 1) * item_width, y)
            end
        end
    end]]
end

dofile_once("data/scripts/perks/perk.lua")

function pvp.move_next_hm(died)
    local inv = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "InventoryGuiComponent")
    EntitySetComponentIsEnabled(ctx.my_player.entity, inv, false)
    hm_y = hm_ys[math.min(pvp.floor, #hm_ys)]
    tp(hm_x - 677, hm_y)
    if pvp.floor > #hm_ys then
        local x, y = hm_x - 480, 10564
        EntityLoad("data/entities/items/pickup/heart_fullhp_temple.xml", x - 16, y)
        EntityLoad("data/entities/items/pickup/spell_refresh.xml", x + 16, y)
        x, y = hm_x - 32, 10626
        perk_spawn_many(x, y)
        spawn_items(hm_x - 327, 10611)
    end
    if died then
        rpc.died(pvp.floor)
        if pvp.last_damage ~= nil then
            local wallet = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "WalletComponent")
            if wallet ~= nil then
                local gold = ComponentGetValue2(wallet, "money")
                local rt = ctx.proxy_opt.pvp_kill_steal / 100
                rpc.give_gold(pvp.last_damage, gold * rt)
                if not ctx.proxy_opt.dont_steal then
                    ComponentSetValue2(wallet, "money", gold * (1 - rt))
                    if math.ceil(gold * rt) ~= 0 then
                        GamePrint(
                            "lost "
                                .. tostring(math.ceil(gold * rt))
                                .. " gold from dying to: "
                                .. ctx.players[pvp.last_damage].name
                        )
                    end
                end
            end
        end
    end
    rpc.remove_floor(pvp.floor)
    pvp.floor = pvp.floor + 1
    pvp.last_damage = nil
    GlobalsSetValue("ew_floor", tostring(pvp.floor))
    local n = pvp.floor % #names_by_floor
    if n == 0 then
        n = #names_by_floor
    end
    if table.contains(needs_ase, names_by_floor[n]) then
        local has_ase = false
        for _, ent in ipairs(EntityGetAllChildren(ctx.my_player.entity)) do
            local com = EntityGetFirstComponentIncludingDisabled(ent, "GameEffectComponent")
            if com ~= nil and ComponentGetValue2(com, "effect") == "REMOVE_FOG_OF_WAR" then
                has_ase = true
                break
            end
        end
        if not has_ase then
            temp_ase = LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/pvp/ase.xml")
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
    async(function()
        wait(1)
        EntitySetComponentIsEnabled(ctx.my_player.entity, inv, true)
    end)
    create_board()
end

function pvp.teleport_into_biome()
    for _, ent in ipairs(boards) do
        EntityKill(ent)
    end
    boards = {}
    rpc.add_floor(pvp.floor, true, true)
    local n = pvp.floor % #chunks_by_floor
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
    LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/pvp/safe_effect.xml")
    async(function()
        wait(8)
        LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/pvp/tp.xml")
        float()
        wait(16)
        x, y = EntityGetTransform(ctx.my_player.entity)
        LoadPixelScene("mods/quant.ew/files/system/pvp/tp.png", "", x - 6, y - 6, "", true, true)
        wait(16)
        x, y = EntityGetTransform(ctx.my_player.entity)
        local did_hit_down, _, _ = RaytracePlatforms(x, y, x, y + 2)
        if did_hit_down then
            LoadGameEffectEntityTo(ctx.my_player.entity, "mods/quant.ew/files/system/pvp/tp.xml")
            float()
            wait(16)
            x, y = EntityGetTransform(ctx.my_player.entity)
            LoadPixelScene("mods/quant.ew/files/system/pvp/tp.png", "", x - 6, y - 6, "", true, true)
        end
    end)
    hm_y = nil
end

local gui

if ctx.proxy_opt.timed then
    gui = GuiCreate()
end

local time = 0

rpc.opts_everywhere()
rpc.opts_reliable()
function rpc.update_timer(t)
    time = t
end

rpc.opts_everywhere()
rpc.opts_reliable()
function rpc.timer_ended(is_hm)
    if is_hm then
        if hm_y ~= nil then
            pvp.teleport_into_biome()
        end
    elseif hm_y == nil then
        pvp.move_next_hm(false)
    end
end

local is_hm

function pvp.on_world_update_host()
    if ctx.proxy_opt.timed then
        if tmr == nil then
            if hm_y ~= nil then
                tmr = 60 * ctx.proxy_opt.time_in
                is_hm = true
            else
                tmr = 60 * ctx.proxy_opt.time_out
                is_hm = false
            end
            rpc.update_timer(tmr)
        else
            tmr = tmr - 1
            if tmr == 0 then
                rpc.timer_ended(is_hm)
                tmr = nil
            elseif GameGetFrameNum() % 20 == 14 then
                rpc.update_timer(tmr)
            end
        end
    end
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
        pvp.floor = tonumber(GlobalsGetValue("ew_floor", "1")) or 1
        my_wins = tonumber(GlobalsGetValue("ew_wins", "0")) or 0
        if my_num == -1 then
            if ctx.is_host then
                my_num = 0
                my_pw = 1
                player_count = 1
                hm_x = BiomeMapGetSize() * 512
            else
                rpc.get_player_num()
            end
            pvp.teleport_into_biome()
        end
        if ctx.proxy_opt.timed then
            GuiStartFrame(gui)
        end
    end
    if ctx.proxy_opt.timed then
        GuiText(gui, 128, 0, math.ceil(time / 60))
    end
    local x, y = EntityGetTransform(ctx.my_player.entity)
    if
        hm_y ~= nil
        and (
            math.floor(hm_y / 512) ~= math.floor(y / 512)
            or math.abs(math.floor(hm_x / 512) - math.floor(x / 512)) > 8
        )
    then
        local has_alive = false
        for _, _ in pairs(pvp.players_by_floor[pvp.floor - 1] or {}) do
            has_alive = true
            break
        end
        if ctx.proxy_opt.wait_on_players and has_alive then
            pvp.floor = pvp.floor - 1
            pvp.move_next_hm(false)
        else
            pvp.teleport_into_biome()
        end
    end
    if player_died[pvp.floor] == nil then
        player_died[pvp.floor] = {}
    end
    if not ctx.proxy_opt.wait_for_time then
        local dead = 0
        for _, _ in pairs(player_died[pvp.floor]) do
            dead = dead + 1
        end
        if
            player_count ~= 1
            and GameGetFrameNum() % 60 == 32
            and #player_died[pvp.floor] == player_count - 1
            and not table.contains(player_died[pvp.floor], ctx.my_id)
        then
            pvp.move_next_hm(false)
            my_wins = my_wins + 1
            GlobalsSetValue("ew_wins", tostring(my_wins))
            rpc.win(my_wins)
        end
    end
    if ctx.proxy_opt.timed and ctx.is_host then
        local n = 0
        for _, d in pairs(pvp.players_by_floor) do
            for _, _ in pairs(d) do
                n = n + 1
            end
        end
        if is_hm then
            if n == player_count then
                tmr = nil
            end
        elseif n == 0 then
            tmr = nil
        end
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
