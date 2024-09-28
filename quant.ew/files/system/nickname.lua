local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")

local rpc = net.new_rpc_namespace()

local util = dofile_once("mods/quant.ew/files/core/util.lua")

local nickname = {}

function nickname.parse( font_filename )
    local id_width = {}

    local file = ModTextFileGetContent(font_filename)
    local quad_open = false
    local space_open = false

    local id = 0
    local width = 0

    for k, split in ipairs(util.string_split(file, "%s")) do
      --print(string.sub(line, 2, 9))

      if (split == "</QuadChar>") then
        quad_open = false
        id_width[id] = width
      end
      if (split == "</WordSpace>") then
        space_open = false
      end

      if (space_open == true) then
        id_width["space"] = tonumber(split)
      end

      if (quad_open == true) then
        if (string.sub(split, 1, 3) == "id=") then
            id = tonumber(string.sub(split, 5, -2))
            --id = string.sub(v, 5, -2)
        end
        if (string.sub(split, 1, 6) == "width=") then
            width = tonumber(string.sub(split, 8, -2))
            --width = string.sub(v, 8, -2)
        end
      end

      if (split == "<QuadChar") then
        quad_open = true
      end

      if (split == "<WordSpace>") then
        space_open = true
      end
    end

    return id_width
end

function nickname.calculate_textwidth(text, font)
    local textwidth = 0

    for i = 1,string.len(text),1
    do
        local l = string.sub( text, i, i)
        if (l == " ") then
            textwidth = textwidth + font["space"]
        else
            local c_id = string.byte(l)
            textwidth = textwidth + (font[c_id] or 1)
        end
    end

    return textwidth
end

function nickname.add_label(player_entity, text, font_filename, scale, font)
    if not EntityGetIsAlive(player_entity) then
        return
    end
    local prev_nickname = EntityGetFirstComponentIncludingDisabled(player_entity, "SpriteComponent", "ew_nickname")
    if prev_nickname ~= nil then
      EntityRemoveComponent(player_entity, prev_nickname)
    end

    if (scale == nil) then
        scale = 1
    end
    if (font == nil) then
        font = nickname.parse(font_filename)
    end
    local textwidth = nickname.calculate_textwidth(text, font)

    local nickname_component = EntityAddComponent2(player_entity, "SpriteComponent", {
        _tags="ew_nickname",
        image_file=font_filename,
        is_text_sprite=true,
        offset_x=textwidth*0.5,
        offset_y=13 + 12/scale,
        update_transform=true,
        update_transform_rotation=false,
        fog_of_war_hole = true,
        text=text,
        z_index=1,
        alpha=0.5,
        emissive = true,
        has_special_scale=true,
        special_scale_x = scale,
        special_scale_y = scale,
    })

    return nickname_component

end

function nickname.on_local_player_spawn(my_player)
  if ctx.proxy_opt.name ~= nil then
    my_player.name = ctx.proxy_opt.name
  end
end

function nickname.on_client_spawned(peer_id, player_data)
  nickname.add_label(player_data.entity, player_data.name, "data/fonts/font_pixel_white.xml", 0.75)
end

function nickname.on_should_send_updates()
  print("Should send nickname update")
  if ctx.proxy_opt.name ~= nil then
    print("Sending name "..ctx.proxy_opt.name)
    rpc.send_name(ctx.proxy_opt.name)
  end
end

rpc.opts_reliable()
function rpc.send_name(name)
  ctx.rpc_player_data.name = name
  nickname.add_label(ctx.rpc_player_data.entity, name, "data/fonts/font_pixel_white.xml", 0.75)
end

return nickname