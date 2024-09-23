local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")

local gui = GuiCreate()

local module = {}

local last_coords = {}

-- "Borrowed" from MK VIII QF 2-puntaa NAVAL-ASE in Noita discord server.
-- https://discord.com/channels/453998283174576133/632303734877192192/1178002118368559175
local function world2gui( x, y )
    in_camera_ref = in_camera_ref or false

    local gui_n = GuiCreate()
    GuiStartFrame(gui_n)
    local w, h = GuiGetScreenDimensions(gui_n)
    GuiDestroy(gui_n)

    local vres_scaling_factor = w/( MagicNumbersGetValue( "VIRTUAL_RESOLUTION_X" ) + MagicNumbersGetValue( "VIRTUAL_RESOLUTION_OFFSET_X" ))
    local cam_x, cam_y = GameGetCameraPos()
    x, y = w/2 + vres_scaling_factor*( x - cam_x ), h/2 + vres_scaling_factor*( y - cam_y )

    return x, y, vres_scaling_factor
end

local function is_suitable_target(entity)
    return EntityGetIsAlive(entity) and not EntityHasTag(entity,"ew_notplayer")
end

function module.on_world_update()
    if EntityHasTag(ctx.my_player.entity, "polymorphed") and not EntityHasTag(ctx.my_player.entity, "ew_notplayer") then
        return
    end
    GuiStartFrame(gui)

    GuiZSet(gui, 10)

    local ccx, ccy = GameGetCameraPos()
    local csx, csy, tcw, tch = GameGetCameraBounds()

    local cw = tcw - 10
    local ch = tch - 10
    local half_cw = cw / 2
    local half_ch = ch / 2

    local gui_id = 2

    for peer_id, player_data in pairs(ctx.players) do
        local px, py = EntityGetTransform(player_data.entity)
        if px == nil then
            if last_coords[peer_id] == nil then
                return
            else
                px, py = last_coords[peer_id][1], last_coords[peer_id][2]
            end
        else
            last_coords[peer_id] = {px, py}
        end
        local player_dir_x = px - ccx
        local player_dir_y = py - ccy
        if player_dir_x > 0 then
            player_dir_x = player_dir_x - 6
        else
            player_dir_x = player_dir_x + 6
        end

        if player_dir_y > 0 then
            player_dir_y = player_dir_y - 6
        else
            player_dir_y = player_dir_y + 6
        end

        local dist_sq = player_dir_x * player_dir_x + player_dir_y * player_dir_y
        -- local dist_sq = player_dir_x * player_dir_x + player_dir_y * player_dir_y
        -- player_dir_x = player_dir_x / dist
        -- player_dir_y = player_dir_y / dist

        local okay_to_display = false

        -- Contain the arrow in screen rect.
        if player_dir_x > half_cw then
            player_dir_y = player_dir_y / (player_dir_x / half_cw)
            player_dir_x = half_cw
            okay_to_display = true
        end
        if player_dir_x < -half_cw then
            player_dir_y = player_dir_y / (player_dir_x / -half_cw)
            player_dir_x = -half_cw
            okay_to_display = true
        end
        if player_dir_y > half_ch then
            player_dir_x = player_dir_x / (player_dir_y / half_ch)
            player_dir_y = half_ch
            okay_to_display = true
        end
        if player_dir_y < -half_ch then
            player_dir_x = player_dir_x / (player_dir_y / -half_ch)
            player_dir_y = -half_ch
            okay_to_display = true
        end

        if okay_to_display then
            local is_host = ctx.host_id == player_data.peer_id
            local is_notplayer = false
            if player_data.status and not player_data.status.is_alive then
                is_notplayer = true
            end
            if not is_notplayer and EntityGetIsAlive(player_data.entity) and EntityHasTag(player_data.entity, "polymorphed_player") then
                goto continue
            end
            if is_notplayer and GameHasFlagRun("ending_game_completed") then
                goto continue
            end
            local x, y = world2gui(ccx+player_dir_x, ccy+player_dir_y)
            local img_path
            if is_host then
                if is_notplayer then
                    img_path = "mods/quant.ew/files/system/player_arrows/arrow_host_notplayer.png"
                else
                    img_path = "mods/quant.ew/files/system/player_arrows/arrow_host.png"
                end
            else
                if is_notplayer then
                    img_path = "mods/quant.ew/files/system/player_arrows/arrow_notplayer.png"
                else
                    img_path = "mods/quant.ew/files/system/player_arrows/arrow.png"
                end
            end
            local scale = math.max(1 / 6, 0.7 - math.atan((math.sqrt(dist_sq) - tch) / 1280) / math.pi)
            GuiImage(gui, gui_id, x, y, img_path, 1, scale, 0, math.atan2(player_dir_y, player_dir_x) + math.pi/2)
            gui_id = gui_id + 1
        end

        ::continue::
    end
end

return module