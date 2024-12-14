local mod = {}

local alive = {}

local last = 0

local hole_last = {}

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

local function hole(item)
    local ce = EntityGetFirstComponent(item, "CellEaterComponent")
    if ce == nil or ComponentGetValue2(ce, "only_stain") then
        return
    end
    local x, y = EntityGetTransform(item)
    local n = ComponentGetValue2(ce, "radius")
    local lx, ly
    if hole_last[item] ~= nil then
        lx, ly = hole_last[item].last[1], hole_last[item].last[2]
        local nx, ny = lx, ly
        if hole_last[item].slast ~= nil then
            nx, ny = hole_last[item].slast[1], hole_last[item].slast[2]
        end
        local inp = math.floor(x).." "..math.floor(nx)
                .." "..math.floor(y).." "..math.floor(ny) .. " " .. n
        net.proxy_send("cut_through_world_line", inp)
    end
    hole_last[item] = {last = {x, y}}
    if lx ~= nil then
        hole_last[item].slast = {lx, ly}
    end
    return true
end

local function update(ent)
    local proj = EntityGetFirstComponentIncludingDisabled(ent, "ProjectileComponent")
    if proj ~= nil and (ComponentGetValue2(proj, "on_death_explode") or ComponentGetValue2(proj, "on_lifetime_out_explode")) then
        local x, y = EntityGetTransform(ent)
        local r = ComponentObjectGetValue2(proj, "config_explosion", "explosion_radius")
        if alive[ent] == nil then
            alive[ent] = {}
        end
        alive[ent].expl = {x, y, r,
                           ComponentObjectGetValue2(proj, "config_explosion", "max_durability_to_destroy"),
                           ComponentObjectGetValue2(proj, "config_explosion", "ray_energy")}

    end
    local mat = EntityGetFirstComponent(ent, "MagicConvertMaterialComponent")
    if mat ~= nil and ComponentGetValue2(mat, "from_material_tag") == "[solid]" then
        local x, y = EntityGetTransform(ent)
        if alive[ent] == nil then
            alive[ent] = {}
        end
        alive[ent].del = {x, y, ComponentGetValue2(mat, "radius"), ComponentGetValue2(mat, "to_material")}
    end
    if hole(ent) ~= nil then
        if alive[ent] == nil then
            alive[ent] = {}
        end
        alive[ent].eater = true
    end
end

function mod.on_world_update_host()
    if first then
        send_mats()
        first = false
    end
    local count = tonumber(ModSettingGet("quant.ew.explosions"))
    for ent, data in pairs(alive) do
        if not EntityGetIsAlive(ent) then
            if count > 0 then
                if data.expl ~= nil then
                    count = count - data.expl[3]
                    local inp = math.floor(data.expl[1]) .. " " .. math.floor(data.expl[2])
                            .. " " .. math.floor(data.expl[3]) .. " " .. math.floor(data.expl[4]) .. " " .. math.floor(data.expl[5])
                    net.proxy_send("cut_through_world_explosion", inp)
                end
                if data.del ~= nil then
                    count = count - data.del[3]
                    local inp = math.floor(data.del[1]) .. " " .. math.floor(data.del[2])
                            .. " " .. math.floor(data.del[3]) .. " " .. math.floor(data.del[4])
                    net.proxy_send("cut_through_world_circle", inp)
                end
                alive[ent] = nil
                hole_last[ent] = nil
            end
        else
            update(ent)
        end
    end
    local n = EntitiesGetMaxID()
    for ent = last + 1, n do
        if EntityGetIsAlive(ent) then
            update(ent)
        end
    end
    last = n
end

return mod