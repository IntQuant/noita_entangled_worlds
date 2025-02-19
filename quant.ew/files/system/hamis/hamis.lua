local rpc = net.new_rpc_namespace()

ModLuaFileAppend("data/scripts/animals/longleg_pet.lua", "mods/quant.ew/files/system/hamis/append.lua")

local function frame()
    if ctx.my_id == ctx.host_id then
        return GameGetFrameNum()
    else
        return ctx.host_frame_num
    end
end

rpc.opts_everywhere()
function rpc.pet(entity_who_interacted, gid, num, hx, hy)
    local entity_interacted = ewext.find_by_gid(gid)
    if entity_interacted == nil then
        return
    end
    entity_who_interacted = entity_who_interacted.entity or entity_who_interacted
    local x, y = EntityGetTransform(entity_interacted)

    edit_component(entity_interacted, "VelocityComponent", function(comp, vars)
        ComponentSetValueVector2(comp, "mVelocity", 0, 0)
    end)

    edit_component(entity_interacted, "CharacterDataComponent", function(comp, vars)
        ComponentSetValueVector2(comp, "mVelocity", 0, 0)
    end)

    edit_component(entity_who_interacted, "VelocityComponent", function(comp, vars)
        ComponentSetValueVector2(comp, "mVelocity", 0, 0)
    end)

    edit_component(entity_who_interacted, "CharacterDataComponent", function(comp, vars)
        ComponentSetValueVector2(comp, "mVelocity", 0, 0)
    end)

    SetRandomSeed(hx, hy + num)
    rnd = Random(1, 20)

    if rnd ~= 13 then
        GamePlayAnimation(entity_interacted, "pet", 99, "stand", 0)
        EntitySetComponentsWithTagEnabled(entity_interacted, "enabled_if_charmed", false)

        GamePrint("$ui_longleg_love_msg1")
    else
        EntityLoad("data/entities/projectiles/explosion.xml", x, y)

        GamePrint("$ui_longleg_love_msg2")
    end

    GameEntityPlaySound(entity_who_interacted, "pet")
end

util.add_cross_call("ew_pet_hamis", function(x, ent)
    local hx, hy = EntityGetTransform(ent)
    local gid
    for _, v in ipairs(EntityGetComponent(ent, "VariableStorageComponent") or {}) do
        if ComponentGetValue2(v, "name") == "ew_gid_lid" then
            gid = v
            break
        end
    end
    if gid ~= nil then
        rpc.pet(ctx.player_data_by_local_entity[x] or x, ComponentGetValue2(gid, "value_string"), frame(), hx, hy)
    end
end)

rpc.opts_everywhere()
rpc.opts_reliable()
function rpc.run_fungus(file, x, y)
    EntityLoad(file, x, y)
end

util.add_cross_call("ew_run_fungus", function(file, x, y)
    rpc.run_fungus(file, x, y)
end)

return {}
