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
function rpc.pet(entity_who_interacted, entity_interacted, num, hx, hy)
    local rnd = entity_interacted
    if not ctx.is_host then
        entity_interacted = ctx.entity_by_remote_id[entity_interacted].id
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

    SetRandomSeed(hx + rnd, hy + num)
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

util.add_cross_call("ew_pet_hamis", function(x, y)
    local hx, hy = EntityGetTransform(y)
    local ent = y
    for a, b in pairs(ctx.entity_by_remote_id) do
        if b.id == y then
            ent = a
            break
        end
    end
    rpc.pet(ctx.player_data_by_local_entity[x] or x, ent, frame(), hx, hy)
end)

return {}
