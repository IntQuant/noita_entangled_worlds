local util = dofile_once("mods/quant.ew/files/src/util.lua")

local nickname = {}
  
function nickname.parse( font_filename )
    local id_width = {}

    local file = ModTextFileGetContent(font_filename)
    --GamePrint(file:find("QuadChar"))
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
        --GamePrint(split)
        id_width["space"] = tonumber(split)
      end

      if (quad_open == true) then
        --GamePrint(split)
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
            --GamePrint("Char: ".. l .. ". Id: "..tostring(c_id))
            textwidth = textwidth + font[c_id]
        end
    end

    return textwidth
end

function nickname.addLabel(player_entity, text, font_filename, scale, font)
    
    if (scale == nil) then
        scale = 1
    end
    if (font == nil) then
        font = nickname.parse(font_filename)
    end
    local textwidth = nickname.calculate_textwidth(text, font)

    local nickname_component = EntityAddComponent2(player_entity, "SpriteComponent", {
        image_file=font_filename,
        is_text_sprite=true,
        offset_x=textwidth*0.5,
        offset_y=13 + 12/scale,
        update_transform=true,
        update_transform_rotation=false,
        fog_of_war_hole = true,
        text=text,
        z_index="1",
        alpha=0.5,
        emissive = true,
        has_special_scale=true,
        special_scale_x = scale,
        special_scale_y = scale,
    })

    return nickname_component

end

return nickname