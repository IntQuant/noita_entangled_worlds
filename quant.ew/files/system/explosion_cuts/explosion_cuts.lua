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
        local hp = element.attr.hp or 100
        local dur = element.attr.durability or 0
        local cell_type = element.attr.cell_type or "liquid"
        local liquid_sand = element.attr.liquid_sand or false
        local liquid_static = element.attr.liquid_static or false
        if element.name == "CellDataChild" then
            local p = info[element.attr._parent]
            if p ~= nil then
                dur = element.attr.durability or p[1]
                hp = element.attr.hp or p[2]
                cell_type = element.attr.cell_type or p[3]
                liquid_sand = element.attr.liquid_sand or p[4]
                liquid_static = element.attr.liquid_static or p[5]
            end
        elseif element.name ~= "CellData" then
            goto continue
        end
        info[element.attr.name] = {dur, hp, cell_type, liquid_sand, liquid_static}
        inp = inp .. mats[element.attr.name] .. " "
                .. dur .. " " .. hp .. " "
                .. cell_type .. " " .. liquid_sand .. " " .. liquid_static
        ::continue::
    end
    net.proxy_send("material_list", string.sub(inp, 0, -2))
end

local first = true

local function hole(item)
    local ce = EntityGetFirstComponent(item, "CellEaterComponent")
    if ce == nil or ComponentGetValue2(ce, "only_stain")
            or ComponentGetValue2(ce, "limited_materials")
            or ComponentGetValue2(ce, "eat_probability") < 40 then
        return
    end
    local r = 0
    local x, y = EntityGetTransform(item)
    local n = ComponentGetValue2(ce, "radius")
    if hole_last[item] ~= nil then
        local lx, ly = hole_last[item][1], hole_last[item][2]
        if lx ~= x or y ~= ly then
            local dx = x - lx
            local dy = y - ly
            r = math.sqrt(dx * dx + dy * dy)
            local inp = math.floor(x).." "..math.floor(lx)
                    .." "..math.floor(y).." "..math.floor(ly) .. " " .. n
            net.proxy_send("cut_through_world_line", inp)
        end
    else
        local inp = math.floor(x) .. " " .. math.floor(y)
                .. " " .. math.floor(n) .. " " .. math.floor(0)
        net.proxy_send("cut_through_world_circle", inp)
    end
    hole_last[item] = {x, y}
    return r
end

local function update(ent)
    if ctx.is_host then
        local proj = EntityGetFirstComponentIncludingDisabled(ent, "ProjectileComponent")
        if proj ~= nil and (ComponentGetValue2(proj, "on_death_explode") or ComponentGetValue2(proj, "on_lifetime_out_explode")) then
            local x, y = EntityGetTransform(ent)
            local r = ComponentObjectGetValue2(proj, "config_explosion", "explosion_radius")
            if r > 4 then
                if alive[ent] == nil then
                    alive[ent] = {}
                end
                alive[ent].expl = {x, y, r,
                                   ComponentObjectGetValue2(proj, "config_explosion", "max_durability_to_destroy"),
                                   ComponentObjectGetValue2(proj, "config_explosion", "ray_energy")}
            elseif alive[ent] ~= nil and alive[ent].expl ~= nil then
                alive[ent].expl = nil
            end
        end
    end
        local mat = EntityGetFirstComponent(ent, "MagicConvertMaterialComponent")
        if mat ~= nil and ComponentGetValue2(mat, "from_material_tag") == "[solid]" then
            local x, y = EntityGetTransform(ent)
            if alive[ent] == nil then
                alive[ent] = {}
            end
            alive[ent].del = {x, y, ComponentGetValue2(mat, "radius"), ComponentGetValue2(mat, "to_material")}
        end
    local l = hole(ent)
    if l ~= nil then
        if alive[ent] == nil then
            alive[ent] = {}
        end
        alive[ent].eater = true
    end
    return l or 0
end

function mod.on_world_update()
    if first then
        send_mats()
        first = false
    end
    local count1 = tonumber(ModSettingGet("quant.ew.explosions") or 128)
    local count2 = tonumber(ModSettingGet("quant.ew.cell_eater") or 64)
    for ent, data in pairs(alive) do
        if not EntityGetIsAlive(ent) then
            if count1 > 0 then
                if data.expl ~= nil then
                    count1 = count1 - data.expl[3]
                    local inp = math.floor(data.expl[1]) .. " " .. math.floor(data.expl[2])
                            .. " " .. math.floor(data.expl[3]) .. " " .. math.floor(data.expl[4]) .. " " .. math.floor(data.expl[5])
                    net.proxy_send("cut_through_world_explosion", inp)
                end
                if data.del ~= nil then
                    count1 = count1 - data.del[3]
                    local inp = math.floor(data.del[1]) .. " " .. math.floor(data.del[2])
                            .. " " .. math.floor(data.del[3]) .. " " .. math.floor(data.del[4])
                    net.proxy_send("cut_through_world_circle", inp)
                end
                alive[ent] = nil
                hole_last[ent] = nil
            end
        elseif count2 > 0 then
            count2 = count2 - update(ent)
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