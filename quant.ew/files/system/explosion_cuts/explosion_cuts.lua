local mod = {}

local alive = {}

local last = 0

local hole_last = {}

local nxml = dofile_once("mods/quant.ew/files/lib/nxml.lua")

local mats = {}

local rpc = net.new_rpc_namespace()

local function send_mats()
    local content_materials = ModTextFileGetContent("data/materials.xml")
    local xml_orig = nxml.parse(content_materials)
    local inp = ""
    local i = 0
    local name = CellFactory_GetName(i)
    while name ~= "unknown" do
        mats[name] = i
        i = i + 1
        name = CellFactory_GetName(i)
    end
    local info = {}
    for element in xml_orig:each_child() do
        if element.name == "CellDataChild" or element.name == "CellData" then
            local hp = element.attr.hp or 100
            local dur = element.attr.durability or 0
            local cell_type = element.attr.cell_type or "liquid"
            local liquid_sand = element.attr.liquid_sand or 0
            local liquid_static = element.attr.liquid_static or 0
            if element.attr._parent ~= nil then
                local p = info[element.attr._parent]
                if p ~= nil then
                    dur = element.attr.durability or p[1]
                    hp = element.attr.hp or p[2]
                    cell_type = element.attr.cell_type or p[3]
                    liquid_sand = element.attr.liquid_sand or p[4]
                    liquid_static = element.attr.liquid_static or p[5]
                end
            end
            info[element.attr.name] = { dur, hp, cell_type, liquid_sand, liquid_static }
            inp = inp
                .. mats[element.attr.name]
                .. " "
                .. dur
                .. " "
                .. hp
                .. " "
                .. cell_type
                .. " "
                .. tostring(liquid_sand)
                .. " "
                .. tostring(liquid_static)
                .. " "
        end
    end
    net.proxy_send("material_list", string.sub(inp, 0, -2))
end

local first = true

local function hole(item)
    local ce = EntityGetFirstComponent(item, "CellEaterComponent")
    if ce == nil or ComponentGetValue2(ce, "only_stain") or ComponentGetValue2(ce, "limited_materials") then
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
            local inp = math.floor(x)
                .. " "
                .. math.floor(lx)
                .. " "
                .. math.floor(y)
                .. " "
                .. math.floor(ly)
                .. " "
                .. n
                .. " "
                .. math.floor(ComponentGetValue2(ce, "eat_probability"))
            net.proxy_send("cut_through_world_line", inp)
        end
    else
        local inp = math.floor(x)
            .. " "
            .. math.floor(y)
            .. " "
            .. math.floor(n)
            .. " "
            .. math.floor(0)
            .. " "
            .. math.floor(ComponentGetValue2(ce, "eat_probability"))
        net.proxy_send("cut_through_world_circle", inp)
    end
    hole_last[item] = { x, y }
    return r
end

local function update(ent, count)
    local proj = EntityGetFirstComponentIncludingDisabled(ent, "ProjectileComponent")
    if
        proj ~= nil
        and (ComponentGetValue2(proj, "on_death_explode") or ComponentGetValue2(proj, "on_lifetime_out_explode"))
    then
        local x, y = EntityGetTransform(ent)
        local r = ComponentObjectGetValue2(proj, "config_explosion", "explosion_radius")
        if r > 4 then
            if alive[ent] == nil then
                alive[ent] = {}
            end
            alive[ent].expl = {
                x,
                y,
                r,
                ComponentObjectGetValue2(proj, "config_explosion", "max_durability_to_destroy"),
                ComponentObjectGetValue2(proj, "config_explosion", "ray_energy"),
                ComponentObjectGetValue2(proj, "config_explosion", "hole_enabled"),
                ComponentObjectGetValue2(proj, "config_explosion", "hole_destroy_liquid"),
                ComponentObjectGetValue2(proj, "config_explosion", "create_cell_material"),
                ComponentObjectGetValue2(proj, "config_explosion", "create_cell_probability"),
            }
        elseif alive[ent] ~= nil and alive[ent].expl ~= nil then
            alive[ent].expl = nil
        end
    end
    local mat = EntityGetFirstComponent(ent, "MagicConvertMaterialComponent")
    if mat ~= nil and ComponentGetValue2(mat, "from_material_tag") == "[solid]" then
        local x, y = EntityGetTransform(ent)
        if alive[ent] == nil then
            alive[ent] = {}
        end
        alive[ent].del = { x, y, ComponentGetValue2(mat, "radius"), ComponentGetValue2(mat, "to_material") }
    end
    local l
    if count > 0 then
        l = hole(ent)
        if l ~= nil then
            if alive[ent] == nil then
                alive[ent] = {}
            end
            alive[ent].eater = true
        end
    end
    return l or 0
end

function rpc.check_mats(new_mats, ping)
    local is_true = false
    for a, b in pairs(mats) do
        if b ~= new_mats[a] then
            GamePrint("MATERIALS DIFFER BETWEEN YOU AND " .. ctx.rpc_player_data.name .. ", CHECK MOD ORDER")
            is_true = true
        end
    end
    if is_true and ping then
        rpc.check_mats(mats)
    end
end

local exists

function mod.on_world_update()
    if first then
        send_mats()
        rpc.check_mats(mats, true)
        first = false
    end
    if not ctx.is_host then
        return
    end
    local count1 = tonumber(ModSettingGet("quant.ew.explosions") or 128)
    local count2 = tonumber(ModSettingGet("quant.ew.cell_eater") or 64)
    for ent, data in pairs(alive) do
        if not EntityGetIsAlive(ent) then
            if count1 > 0 then
                if data.expl ~= nil then
                    count1 = count1 - data.expl[3]
                    local inp = math.floor(data.expl[1])
                        .. " "
                        .. math.floor(data.expl[2])
                        .. " "
                        .. math.floor(data.expl[3])
                        .. " "
                        .. math.floor(data.expl[4])
                        .. " "
                        .. math.floor(data.expl[5])
                        .. " "
                        .. tostring(data.expl[6])
                        .. " "
                        .. tostring(data.expl[7])
                        .. " "
                        .. (mats[data.expl[8] or "air"] or 0)
                        .. " "
                        .. math.floor(data.expl[9])
                    net.proxy_send("cut_through_world_explosion", inp)
                    exists = true
                end
                if data.del ~= nil then
                    count1 = count1 - data.del[3]
                    local inp = math.floor(data.del[1])
                        .. " "
                        .. math.floor(data.del[2])
                        .. " "
                        .. math.floor(data.del[3])
                        .. " "
                        .. math.floor(data.del[4])
                    net.proxy_send("cut_through_world_circle", inp)
                end
                alive[ent] = nil
                hole_last[ent] = nil
            end
        else
            count2 = count2 - update(ent, count2)
        end
    end
    local n = EntitiesGetMaxID()
    for ent = last + 1, n do
        if EntityGetIsAlive(ent) then
            update(ent, 1)
        end
    end
    if exists then
        net.proxy_send("flush_exp", "")
        exists = nil
    end
    last = n
end

return mod
