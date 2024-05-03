local ctx = dofile_once("mods/quant.ew/files/src/ctx.lua")
local inventory_helper = dofile_once("mods/quant.ew/files/src/inventory_helper.lua")

local ffi = require("ffi")

ffi.cdef([[
#pragma pack(push, 1)
typedef struct A {
    float aim_x;
    float aim_y;
    float aimNormal_x;
    float aimNormal_y;
    float aimNonZero_x;
    float aimNonZero_y;
    float mouse_x;
    float mouse_y;
    float mouseRaw_x;
    float mouseRaw_y;
    float mouseRawPrev_x;
    float mouseRawPrev_y;
    float mouseDelta_x;
    float mouseDelta_y;
    bool kick:1;
    bool fire:1;
    bool fire2:1;
    bool action:1;
    bool throw:1;
    bool interact:1;
    bool left:1;
    bool right:1;
    bool up:1;
    bool down:1;
    bool jump:1;
    bool fly:1;
    bool leftClick:1;
    bool rightClick:1;
} Controls;
#pragma pack(pop)
]])
ffi.cdef([[
#pragma pack(push, 1)
typedef struct D {
    int frames_in_air;
    float x;
    float y;
    float vel_x;
    float vel_y;
    bool is_on_ground:1;
    bool is_on_slippery_ground:1;
} CharacterPos;
#pragma pack(pop)
]])
ffi.cdef([[
#pragma pack(push, 1)
typedef struct E {
    float x;
    float y;
    float r;
    int special_seed;
    int player_action_rng;
} FireWand;
#pragma pack(pop)
]])

local Controls = ffi.typeof("Controls")
local CharacterPos = ffi.typeof("CharacterPos")
local FireWand = ffi.typeof("FireWand")

local player_fns = {
    deserialize_inputs = function(message, player_data)
        if (player_data ~= nil and player_data.entity ~= nil and EntityGetIsAlive(player_data.entity)) then
            --print(json.stringify(message))
            
            local controls_data = player_data.controls
            local controlsComp = EntityGetFirstComponentIncludingDisabled(player_data.entity, "ControlsComponent")

            if(message.kick)then
                ComponentSetValue2(controlsComp, "mButtonDownKick", true)
                if (not controls_data.kick) then
                    ComponentSetValue2(controlsComp, "mButtonFrameKick", GameGetFrameNum() + 1)
                end
                controls_data.kick = true
            else
                ComponentSetValue2(controlsComp, "mButtonDownKick", false)
                controls_data.kick = false
            end

            --EntityHelper.BlockFiring(data.players[tostring(user)].entity, true)

            if(message.fire)then
                ComponentSetValue2(controlsComp, "mButtonDownFire", true)
                if (not controls_data.fire) then
                    ComponentSetValue2(controlsComp, "mButtonFrameFire", GameGetFrameNum()+1)
                end
                ComponentSetValue2(controlsComp, "mButtonLastFrameFire", GameGetFrameNum())
                controls_data.fire = true
            else
                ComponentSetValue2(controlsComp, "mButtonDownFire", false)
                controls_data.fire = false
            end

            if(message.fire2)then
                ComponentSetValue2(controlsComp, "mButtonDownFire2", true)
                if (not controls_data.fire2) then
                    ComponentSetValue2(controlsComp, "mButtonFrameFire2", GameGetFrameNum()+1)
                end
                controls_data.fire2 = true
            else
                ComponentSetValue2(controlsComp, "mButtonDownFire2", false)
                controls_data.fire2 = false
            end

            if(message.action)then
                ComponentSetValue2(controlsComp, "mButtonDownAction", true)
                if (not controls_data.action) then
                    ComponentSetValue2(controlsComp, "mButtonFrameAction", GameGetFrameNum() + 1)
                end
                controls_data.action = true
            else
                ComponentSetValue2(controlsComp, "mButtonDownAction", false)
                controls_data.action = false
            end

            if(message.throw)then
                ComponentSetValue2(controlsComp, "mButtonDownThrow", true)
                if (not controls_data.throw) then
                    ComponentSetValue2(controlsComp, "mButtonFrameThrow", GameGetFrameNum() + 1)
                end
                controls_data.throw = true
            else
                ComponentSetValue2(controlsComp, "mButtonDownThrow", false)
                controls_data.throw = false
            end

            if(message.interact)then
                ComponentSetValue2(controlsComp, "mButtonDownInteract", true)
                if (not controls_data.interact) then
                    ComponentSetValue2(controlsComp, "mButtonFrameInteract", GameGetFrameNum() + 1)
                end
                controls_data.interact = true
            else
                ComponentSetValue2(controlsComp, "mButtonDownInteract", false)
                controls_data.interact = false
            end

            if(message.left)then
                ComponentSetValue2(controlsComp, "mButtonDownLeft", true)
                if (not controls_data.left) then
                    ComponentSetValue2(controlsComp, "mButtonFrameLeft", GameGetFrameNum() + 1)
                end
                controls_data.left = true
            else
                ComponentSetValue2(controlsComp, "mButtonDownLeft", false)
                controls_data.left = false
            end

            if(message.right)then
                ComponentSetValue2(controlsComp, "mButtonDownRight", true)
                if (not controls_data.right) then
                    ComponentSetValue2(controlsComp, "mButtonFrameRight", GameGetFrameNum() + 1)
                end
                controls_data.right = true
            else
                ComponentSetValue2(controlsComp, "mButtonDownRight", false)
                controls_data.right = false
            end

            if(message.up)then
                ComponentSetValue2(controlsComp, "mButtonDownUp", true)
                if (not controls_data.up) then
                    ComponentSetValue2(controlsComp, "mButtonFrameUp", GameGetFrameNum() + 1)
                end
                controls_data.up = true
            else
                ComponentSetValue2(controlsComp, "mButtonDownUp", false)
                controls_data.up = false
            end

            if(message.down)then
                ComponentSetValue2(controlsComp, "mButtonDownDown", true)
                if (not controls_data.down) then
                    ComponentSetValue2(controlsComp, "mButtonFrameDown", GameGetFrameNum() + 1)
                end
                controls_data.down = true
            else
                ComponentSetValue2(controlsComp, "mButtonDownDown", false)
                controls_data.down = false
            end

            if(message.jump)then
                ComponentSetValue2(controlsComp, "mButtonDownJump", true)
                if (not controls_data.jump) then
                    ComponentSetValue2(controlsComp, "mButtonFrameJump", GameGetFrameNum() + 1)
                end
                controls_data.jump = true
            else
                ComponentSetValue2(controlsComp, "mButtonDownJump", false)
                controls_data.jump = false
            end

            if(message.fly)then
                ComponentSetValue2(controlsComp, "mButtonDownFly", true)
                if (not controls_data.fly) then
                    ComponentSetValue2(controlsComp, "mButtonFrameFly", GameGetFrameNum() + 1)
                end
                controls_data.fly = true
            else
                ComponentSetValue2(controlsComp, "mButtonDownFly", false)
                controls_data.fly = false
            end

            if(message.leftClick)then
                ComponentSetValue2(controlsComp, "mButtonDownLeftClick", true)
                if (not controls_data.leftClick) then
                    ComponentSetValue2(controlsComp, "mButtonFrameLeftClick", GameGetFrameNum() + 1)
                end
                controls_data.leftClick = true
            else
                ComponentSetValue2(controlsComp, "mButtonDownLeftClick", false)
                controls_data.leftClick = false
            end

            if(message.rightClick)then
                ComponentSetValue2(controlsComp, "mButtonDownRightClick", true)
                if (not controls_data.rightClick) then
                    ComponentSetValue2(controlsComp, "mButtonFrameRightClick", GameGetFrameNum() + 1)
                end
                controls_data.rightClick = true
            else
                ComponentSetValue2(controlsComp, "mButtonDownRightClick", false)
                controls_data.rightClick = false
            end


            --[[
            local aim_x, aim_y = ComponentGetValue2(controls, "mAimingVector") -- float, float
            local aimNormal_x, aimNormal_y = ComponentGetValue2(controls, "mAimingVectorNormalized") -- float, float
            local aimNonZero_x, aimNonZero_y = ComponentGetValue2(controls, "mAimingVectorNonZeroLatest") -- float, float
            local mouse_x, mouse_y = ComponentGetValue2(controls, "mMousePosition") -- float, float
            local mouseRaw_x, mouseRaw_y = ComponentGetValue2(controls, "mMousePositionRaw") -- float, float
            local mouseRawPrev_x, mouseRawPrev_y = ComponentGetValue2(controls, "mMousePositionRawPrev") -- float, float
            local mouseDelta_x, mouseDelta_y = ComponentGetValue2(controls, "mMouseDelta") -- float, float
            ]]

            ComponentSetValue2(controlsComp, "mAimingVector", message.aim_x, message.aim_y)
            ComponentSetValue2(controlsComp, "mAimingVectorNormalized", message.aimNormal_x, message.aimNormal_y)
            ComponentSetValue2(controlsComp, "mAimingVectorNonZeroLatest", message.aimNonZero_x, message.aimNonZero_y)
            ComponentSetValue2(controlsComp, "mMousePosition", message.mouse_x, message.mouse_y)
            ComponentSetValue2(controlsComp, "mMousePositionRaw", message.mouseRaw_x, message.mouseRaw_y)
            ComponentSetValue2(controlsComp, "mMousePositionRawPrev", message.mouseRawPrev_x, message.mouseRawPrev_y)
            ComponentSetValue2(controlsComp, "mMouseDelta", message.mouseDelta_x, message.mouseDelta_y)

            local children = EntityGetAllChildren(player_data.entity) or {}
            for i, child in ipairs(children) do
                if (EntityGetName(child) == "cursor") then
                    --EntitySetTransform(child, message.mouse_x, message.mouse_y)
                    EntityApplyTransform(child, message.mouse_x, message.mouse_y)
                end
            end

        end
    end,
    serialize_inputs = function(player_data)
        local player = player_data.entity
        if (player == nil) then
            return
        end
        local controls = EntityGetFirstComponentIncludingDisabled(player, "ControlsComponent")

        
        if (controls ~= nil) then
            local kick = ComponentGetValue2(controls, "mButtonDownKick") -- boolean
            local fire = ComponentGetValue2(controls, "mButtonDownFire")  -- boolean
            local fire2 = ComponentGetValue2(controls, "mButtonDownFire2")  -- boolean
            local action = ComponentGetValue2(controls, "mButtonDownAction") -- boolean
            local throw = ComponentGetValue2(controls, "mButtonDownThrow") -- boolean
            local interact = ComponentGetValue2(controls, "mButtonDownInteract") -- boolean
            local left = ComponentGetValue2(controls, "mButtonDownLeft") -- boolean
            local right = ComponentGetValue2(controls, "mButtonDownRight") -- boolean
            local up = ComponentGetValue2(controls, "mButtonDownUp") -- boolean
            local down = ComponentGetValue2(controls, "mButtonDownDown") -- boolean
            local jump = ComponentGetValue2(controls, "mButtonDownJump") -- boolean
            local fly = ComponentGetValue2(controls, "mButtonDownFly") -- boolean
            local leftClick = ComponentGetValue2(controls, "mButtonDownLeftClick") -- boolean
            local rightClick = ComponentGetValue2(controls, "mButtonDownRightClick") -- boolean
            local aim_x, aim_y = ComponentGetValue2(controls, "mAimingVector") -- float, float
            local aimNormal_x, aimNormal_y = ComponentGetValue2(controls, "mAimingVectorNormalized") -- float, float
            local aimNonZero_x, aimNonZero_y = ComponentGetValue2(controls, "mAimingVectorNonZeroLatest") -- float, float
            local mouse_x, mouse_y = ComponentGetValue2(controls, "mMousePosition") -- float, float
            local mouseRaw_x, mouseRaw_y = ComponentGetValue2(controls, "mMousePositionRaw") -- float, float
            local mouseRawPrev_x, mouseRawPrev_y = ComponentGetValue2(controls, "mMousePositionRawPrev") -- float, float
            local mouseDelta_x, mouseDelta_y = ComponentGetValue2(controls, "mMouseDelta") -- float, float

            local c = Controls{
                kick = kick,
                fire = fire,
                fire2 = fire2,
                action = action,
                throw = throw,
                interact = interact,
                left = left,
                right = right,
                up = up,
                down = down,
                jump = jump,
                fly = fly,
                leftClick = leftClick,
                rightClick = rightClick,
                aim_x = aim_x,
                aim_y = aim_y,
                aimNormal_x = aimNormal_x,
                aimNormal_y = aimNormal_y,
                aimNonZero_x = aimNonZero_x,
                aimNonZero_y = aimNonZero_y,
                mouse_x = mouse_x,
                mouse_y = mouse_y,
                mouseRaw_x = mouseRaw_x,
                mouseRaw_y = mouseRaw_y,
                mouseRawPrev_x = mouseRawPrev_x,
                mouseRawPrev_y = mouseRawPrev_y,
                mouseDelta_x = mouseDelta_x,
                mouseDelta_y = mouseDelta_y,
            }

            return c
        end
    end,
    make_playerdata_for = function(entity_id)
        GamePrint("Made playerdata for "..entity_id)
        return {
            entity = entity_id,
            controls = {},
        }
    end,
}

function player_fns.serialize_position(player_data)
    local entity = player_data.entity
    if not EntityGetIsAlive(entity) then
        return
    end
    local x, y = EntityGetTransform(entity)
    local character_data = EntityGetFirstComponentIncludingDisabled(entity, "CharacterDataComponent")
    local character_platforming_comp = EntityGetFirstComponentIncludingDisabled(entity, "CharacterPlatformingComponent")
    local vel_x, vel_y = ComponentGetValue2(character_data, "mVelocity")

    local c = CharacterPos{
        frames_in_air = ComponentGetValue2(character_platforming_comp, "mFramesInAirCounter"),
        x = x,
        y = y,
        vx = vel_x,
        vy = vel_y,
        is_on_ground = ComponentGetValue2(character_data, "is_on_ground"),
        is_on_slippery_ground = ComponentGetValue2(character_data, "is_on_slippery_ground"),
    }
    return c
end

function player_fns.deserialize_position(message, player_data)
    local entity = player_data.entity
    if not EntityGetIsAlive(entity) then
        return
    end
    local character_data = EntityGetFirstComponentIncludingDisabled(entity, "CharacterDataComponent")
    local velocity_comp = EntityGetFirstComponentIncludingDisabled(entity, "VelocityComponent")

    ComponentSetValue2(character_data, "mVelocity", message.vel_x, message.vel_y)
    ComponentSetValue2(velocity_comp, "mVelocity", message.vel_x, message.vel_y)

    EntityApplyTransform(entity, message.x, message.y)
end


function player_fns.serialize_items(player_data)
    local item_data, spell_data = inventory_helper.get_item_data(player_data)
    return item_data
end

function player_fns.deserialize_items(inventory_state, player_data)
    inventory_helper.set_item_data(inventory_state, player_data)
end

function player_fns.peer_has_player(peer_id)
    return ctx.players[peer_id] ~= nil
end

function player_fns.peer_get_player_data(peer_id)
    return ctx.players[peer_id]
end

function player_fns.spawn_player_for(peer_id, x, y)
    GamePrint("Spawning player for "..peer_id)
    local new = EntityLoad("mods/quant.ew/files/entities/client.xml", x, y)
    local new_playerdata = player_fns.make_playerdata_for(new)
    ctx.players[peer_id] = new_playerdata
end

function player_fns.is_inventory_open()
    local player_entity = ctx.players[ctx.my_id].entity
    local inventory_gui_comp = EntityGetFirstComponentIncludingDisabled(player_entity, "InventoryGuiComponent")

    return ComponentGetValue2(inventory_gui_comp, "mActive")
end

print("Players initialized")
return player_fns