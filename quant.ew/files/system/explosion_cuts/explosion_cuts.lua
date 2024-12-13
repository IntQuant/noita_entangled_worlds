local mod = {}

local alive = {}

local last = 0

local nxml = dofile_once("mods/quant.ew/files/lib/nxml.lua")

local function send_mats()
    local content_materials = ModTextFileGetContent("data/materials.xml")
    local xml_orig = nxml.parse(content_materials)
    local inp = ""
    local i = 0
    local name = CellFactory_GetName(i)
    local mats = {}
    while name ~= "unknown" do
        mats[name] = i
        i = i + 1
        name = CellFactory_GetName(i)
    end
    local info = {}
    for element in xml_orig:each_child() do
        if element.name == "CellData" then
            local hp = element.attr.hp or 0
            local dur = element.attr.durability or 0
            info[element.attr.name] = {dur, hp}
            inp = inp .. mats[element.attr.name] .. " "
                    .. dur .. " " .. hp .. " "
        elseif element.name == "CellDataChild" then
            local p = info[element.attr._parent]
            if p ~= nil then
                local dur = element.attr.durability or p[1]
                local hp = element.attr.hp or p[2]
                inp = inp .. mats[element.attr.name] .. " "
                        .. dur .. " " .. hp .. " "
            else
                local hp = element.attr.hp or 0
                local dur = element.attr.durability or 0
                inp = inp .. mats[element.attr.name] .. " "
                        .. dur .. " " .. hp .. " "
            end
        end
    end
    net.proxy_send("material_list", string.sub(inp, 0, -2))
end

local first = true

function mod.on_world_update_host()
    if first then
        send_mats()
        first = false
    end
    local n = EntitiesGetMaxID()
    for ent = last + 1, n do
        if EntityGetIsAlive(ent) then
            local proj = EntityGetFirstComponentIncludingDisabled(ent, "ProjectileComponent")
            if proj ~= nil and ComponentGetValue2(proj, "on_death_explode") then
                local x, y = EntityGetTransform(ent)
                alive[ent] = {x, y, ComponentObjectGetValue2(proj, "config_explosion", "explosion_radius"),
                              ComponentObjectGetValue2(proj, "config_explosion", "max_durability_to_destroy"), ComponentObjectGetValue2(proj, "config_explosion", "ray_energy")}
            else
                local mat = EntityGetFirstComponent(ent, "MagicConvertMaterialComponent")
                if mat ~= nil and ComponentGetValue2(mat, "from_material_tag") == "[solid]" then
                    local x, y = EntityGetTransform(ent)
                    alive[ent] = {x, y, ComponentGetValue2(mat, "radius"), ComponentGetValue2(mat, "to_material")}
                end
            end
        end
    end
    last = n
    for ent, data in pairs(alive) do
        if not EntityGetIsAlive(ent) then
            if #alive[ent] == 5 then
                local inp = math.floor(data[1]) .. " " .. math.floor(data[2])
                        .. " " .. math.floor(data[3]) .. " " .. math.floor(data[4]) .. " " .. math.floor(data[5])
                net.proxy_send("cut_through_world_explosion", inp)
            else
                local inp = math.floor(data[1]) .. " " .. math.floor(data[2])
                        .. " " .. math.floor(data[3]) .. " " .. math.floor(data[4])
                net.proxy_send("cut_through_world_circle", inp)
            end
            alive[ent] = nil
        end
    end
end

return mod