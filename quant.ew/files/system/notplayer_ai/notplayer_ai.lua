local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local wandfinder = dofile_once("mods/quant.ew/files/system/notplayer_ai/wandfinder.lua")

local MAX_RADIUS = 128*4

local state

local module = {}

local throw_water = false

local function log(...)
    -- GamePrint(...)
end

local bad_mats = {"magic_liquid_random_polymorph",
                  "magic_liquid_polymorph",
                  "magic_liquid_unstable_polymorph",
                  "acid",
                  "creepy_liquid",
                  "cursed_liquid",
                  "liquid_fire",
                  "liquid_fire_weak",
                  "poison",
                  "just_death",
                  "lava",
                  "pus",
                  "material_confusion",
                  "material_darkness",
                  "mimic_liquid",
                  "void_liquid",
                  "magic_liquid_weakness",
--                "magic_liquid_teleportation",
--                "magic_liquid_unstable_teleportation",
                  "beer",
--                "alcohol",
                  "sima",
                  "blood_cold",
                  "juhannussima",
                  "slime",
                  "slime_yellow",
                  "slime_green"}

local good_mats = {"magic_liquid_movement_faster",
                   "magic_liquid_protection_all",
                   "magic_liquid_berserk",
                   "magic_liquid_mana_regeneration",
                   "magic_liquid_faster_levitation_and_movement",
                   "magic_liquid_hp_regeneration",
--                 "magic_liquid_invisibility",
                   "magic_liquid_faster_levitation",
                   "magic_liquid_hp_regeneration_unstable"}

local water_mats = {"water", "swamp", "water_swamp", "water_salt", "blood", "mud", "snow"}

local ignore_spell = {"ANTIHEAL", "BLACK_HOLE", "BLACK_HOLE_DEATH_TRIGGER", "POWERDIGGER", "DIGGER", "PIPE_BOMB", "PIPE_BOMB_DEATH_TRIGGER", "GRENADE_LARGE", "CRUMBLING_EARTH", "HEAL_BULLET", "FISH",
                      "TELEPORT_PROJECTILE_CLOSER", "TELEPORT_PROJECTILE_STATIC", "SWAPPER_PROJECTILE", "TELEPORT_PROJECTILE", "TELEPORT_PROJECTILE_SHORT", "WHITE_HOLE", "CESSATION", "ADD_TRIGGER",
                      "ADD_TIMER", "ADD_DEATH_TRIGGER", "DIVIDE_2", "DIVIDE_3", "DIVIDE_4", "DIVIDE_10", "GAMMA", "MU", "ALPHA", "OMEGA", "PHI", "SIGMA", "TAU", "SUMMON_PORTAL", "DUPLICATE",
                      "IF_PROJECTILE", "IF_HP", "IF_ENEMY", "IF_HALF", "IF_ELSE", "IF_END", "ALL_SPELLS"}

local function get_potions_of_type(type)
    local potions = {}
    local children = EntityGetAllChildren(ctx.my_player.entity)
    if children == nil then
        return potions
    end
    local items
    for _, child in pairs(children) do
        if EntityGetName(child) == "inventory_quick" then
            items = child
        end
    end
    local is_bad = type == bad_mats
    local is_water = type == water_mats
    local children_items = EntityGetAllChildren(items)
    if children_items == nil then
        return potions
    end
    for _, item in ipairs(children_items or {}) do
        if EntityHasTag(item, "potion") then
            local mat = EntityGetFirstComponent(item, "MaterialInventoryComponent")
            local materials = ComponentGetValue2(mat, "count_per_material_type")
            local total = 0
            for id, amt in pairs(materials or {}) do
                if amt ~= 0 then
                    local name = CellFactory_GetName(id - 1)
                    local use = false
                    for _, n in ipairs(type) do
                        if name == n then
                            use = true
                        end
                    end
                    if use then
                        total = total + amt
                    end
                end
            end
            if total >= 500 or (is_water and total >= 100) then
                table.insert(potions, item)
            end
        elseif is_bad then
            local name = EntityGetFilename(item)
            if EntityHasTag(item, "evil_eye")
                    or EntityHasTag(item, "thunderstone")
                    or EntityHasTag(item, "normal_tablet")
                    or name == "data/entities/items/pickup/physics_die.xml"
                    or name == "data/entities/items/pickup/physics_greed_die.xml" then
                table.insert(potions, item)
            end
        end
    end
    return potions
end

local function is_potion_of_type(item, type)
    local mat = EntityGetFirstComponent(item, "MaterialInventoryComponent")
    if mat == nil then
        return false
    end
    local materials = ComponentGetValue2(mat, "count_per_material_type")
    local other = 0
    local water = 0
    for id, amt in pairs(materials or {}) do
        if amt ~= 0 then
            local name = CellFactory_GetName(id - 1)
            local use = false
            for _, n in ipairs(type) do
                if name == n then
                    use = true
                end
            end
            if use then
                water = water + amt
            else
                other = other + amt
            end
        end
    end
    return water >= other and water ~= 0
end

local do_kick

local function find_new_wand()
    local children = EntityGetAllChildren(state.attack_wand)
    if children == nil then
        table.insert(state.empty_wands, state.attack_wand)
        state.attack_wand = wandfinder.find_attack_wand(state.empty_wands)
        do_kick = true
    else
        local bad_mod = false
        local is_any_not_empty = false
        for _, child in pairs(children or {}) do
            local is_proj = false
            local is_bad_proj = false
            local sprites = EntityGetComponentIncludingDisabled(child, "SpriteComponent")
            for _, sprite in pairs(sprites) do
                local image = ComponentGetValue2(sprite, "image_file")
                if image == "data/ui_gfx/inventory/item_bg_projectile.png"
                        or image == "data/ui_gfx/inventory/item_bg_other.png" then
                    is_proj = true
                    break
                elseif image == "data/ui_gfx/inventory/item_bg_material.png"
                        or image == "data/ui_gfx/inventory/item_bg_static_projectile.png" then
                    is_bad_proj = true
                    break
                end
            end
            if (is_proj or is_bad_proj) and bad_mod then
                bad_mod = false
                goto continue
            end
            local spell = EntityGetFirstComponentIncludingDisabled(child, "ItemActionComponent")
            local spell_name = ComponentGetValue2(spell, "action_id")
            if spell_name == "NOLLA" or spell_name == "ZERO_DAMAGE" then
                bad_mod = true
                goto continue
            end
            local dont_use = false
            for _, name in ipairs(ignore_spell) do
                if name == spell_name then
                    dont_use = true
                    break
                end
            end
            local item = EntityGetFirstComponentIncludingDisabled(child, "ItemComponent")
            if ComponentGetValue2(item, "uses_remaining") ~= 0 and is_proj and not dont_use then
                is_any_not_empty = true
                break
            end
            ::continue::
        end
        if not is_any_not_empty then
            table.insert(state.empty_wands, state.attack_wand)
            state.attack_wand = wandfinder.find_attack_wand(state.empty_wands)
            do_kick = true
        end
    end
end

local function has_pheremoned(entity)
    local com = EntityGetFirstComponentIncludingDisabled(entity, "StatusEffectDataComponent")
    local stains = ComponentGetValue2(com, "stain_effects")
    return stains ~= nil and stains[17] ~= nil and stains[17] >= 0.15
end

local function has_ambrosia(entity)
    local com = EntityGetFirstComponentIncludingDisabled(entity, "StatusEffectDataComponent")
    local stains = ComponentGetValue2(com, "stain_effects")
    return stains ~= nil and stains[24] ~= nil and stains[24] >= 0.15
end

local function needs_douse(entity)
    local damage_model = EntityGetFirstComponentIncludingDisabled(entity, "DamageModelComponent")
    local prot_fire = false
    local prot_toxic = false
    if damage_model ~= nil then
        local hp = ComponentGetValue2(damage_model, "hp")
        local max_hp = ComponentGetValue2(damage_model, "max_hp")
        if hp / max_hp <= 0.05 then
            prot_toxic = true
        end
    end
    for _, ent in ipairs(EntityGetAllChildren(entity) or {}) do
        local com = EntityGetFirstComponent(ent, "GameEffectComponent")
        if com ~= nil then
            local name = ComponentGetValue2(com, "effect")
            if name == "PROTECTION_FIRE" then
                prot_fire = true
            elseif name == "PROTECTION_RADIOACTIVITY" then
                prot_toxic = true
            end
            if prot_toxic and prot_fire then
                break
            end
        end
    end
    for _, ent in ipairs(EntityGetAllChildren(entity) or {}) do
        local com = EntityGetFirstComponent(ent, "GameEffectComponent")
        if com ~= nil then
            local name = ComponentGetValue2(com, "effect")
            if (name == "ON_FIRE" and not prot_fire)
                    or (name == "RADIOACTIVE" and not prot_toxic) then
                return true
            end
        end
    end
    return false
end

local bad_potion
local good_potion
local water_potion

local dont_throw = true
local stop_potion = false

local bathe = false

local function arc_potion(world_x, world_y)
    local arm = EntityGetAllChildren(ctx.my_player.entity, "player_arm_r")[1]
    local ch_x, ch_y = EntityGetHotspot(arm, "hand", true)
    local dx, dy = world_x - ch_x, world_y - ch_y

    ComponentSetValue2(state.control_component, "mMousePosition", world_x, world_y)
    local v = 180
    local g = 156
    dy = -dy
    local is_behind = dx<0
    if is_behind then
        dx = -dx
    end
    local lhs = v*v/(g*dx)
    local interior = v*v*v*v - g*g*dx*dx - 2*g*dy*v*v
    if interior < 0 then
        dont_throw = true
        bad_potion = nil
        stop_potion = true
        return
    end
    local rhs = math.sqrt(interior)/(g*dx)
    local theta1 = lhs+rhs
    local theta2 = lhs-rhs
    local theta
    if dy < 0 then
        if theta1 > theta2 then
            theta = theta2
        else
            theta = theta1
        end
    else
        if theta1 > theta2 then
            theta = theta1
        else
            theta = theta2
        end
    end
    local cos = 1 / math.sqrt(theta*theta+1)
    local sin = theta * cos
    if theta > theta1 or theta > theta2 then
        local t = v * sin / g
        local x = v*t*cos
        local y = v*t*sin-g*t*t/2
        if is_behind then
            x = -x
        end
        local did_hit_1, _, _ = RaytracePlatforms(ch_x, ch_y, ch_x + x, ch_y + y)
        local did_hit_2, _, _ = RaytracePlatforms(ch_x + x, ch_y + y, world_x, world_y)
        if did_hit_1 or did_hit_2 then
            if theta == theta1 then
                theta = theta2
            else
                theta = theta1
            end
            cos = 1 / math.sqrt(theta*theta+1)
            sin = theta * cos
        end
    end
    if is_behind then
        cos = -cos
    end
    ComponentSetValue2(state.control_component, "mAimingVector", cos * 312, -sin * 312)
    ComponentSetValue2(state.control_component, "mAimingVectorNormalized", cos, -sin)
end


local function aim_at(world_x, world_y)
    if good_potion ~= nil then
        ComponentSetValue2(state.control_component, "mAimingVector", 0, 312)
        ComponentSetValue2(state.control_component, "mAimingVectorNormalized", 0, 1)
        ComponentSetValue2(state.control_component, "mMousePosition", world_x, world_y)
        return
    elseif water_potion ~= nil and not throw_water then
        ComponentSetValue2(state.control_component, "mAimingVector", 0, -312)
        ComponentSetValue2(state.control_component, "mAimingVectorNormalized", 0, -1)
        ComponentSetValue2(state.control_component, "mMousePosition", world_x, world_y)
        return
    end

    if bad_potion ~= nil or throw_water then
        arc_potion(world_x, world_y)
    else
        local arm = EntityGetAllChildren(ctx.my_player.entity, "player_arm_r")[1]
        local ch_x, ch_y = EntityGetHotspot(arm, "hand", true)
        local dx, dy = world_x - ch_x, world_y - ch_y

        local dist = math.sqrt(dx * dx + dy * dy)

        ComponentSetValue2(state.control_component, "mAimingVector", dx, dy)

        ComponentSetValue2(state.control_component, "mMousePosition", world_x, world_y)

        if dist > 0 then
            -- ComponentSetValue2(state.control_component, "mAimingVectorNonZeroLatest", dx, dy)
            ComponentSetValue2(state.control_component, "mAimingVectorNormalized", dx/dist, dy/dist)
        end
    end
end

local throw = false

local function fire_wand(enable)
    if state.is_pheremoned ~= -120 then
        local damage = EntityGetFirstComponent(ctx.my_player.entity, "DamageModelComponent")
        if state.is_pheremoned >= ComponentGetValue2(damage, "mLastDamageFrame") then
            if has_pheremoned(ctx.my_player.entity) then
                enable = false
            else
                state.is_pheremoned = -120
            end
        else
            EntityRemoveStainStatusEffect(ctx.my_player.entity, "CHARM")
            state.is_pheremoned = 0
        end
    end
    if bad_potion ~= nil or good_potion ~= nil or throw_water then
        ComponentSetValue2(state.control_component, "mButtonDownFire", false)
        ComponentSetValue2(state.control_component, "mButtonDownFire2", false)
        if dont_throw then
            return
        end
        ComponentSetValue2(state.control_component, "mButtonDownRightClick", true)
        ComponentSetValue2(state.control_component, "mButtonDownThrow", true)
        if not throw then
            ComponentSetValue2(state.control_component, "mButtonFrameRightClick", GameGetFrameNum() + 1)
            ComponentSetValue2(state.control_component, "mButtonFrameThrow", GameGetFrameNum() + 1)
        end
        throw = true
    else
        if water_potion ~= nil then
            enable = true
        end
        ComponentSetValue2(state.control_component, "mButtonDownRightClick", false)
        ComponentSetValue2(state.control_component, "mButtonDownThrow", false)
        ComponentSetValue2(state.control_component, "mButtonDownFire", enable)
        ComponentSetValue2(state.control_component, "mButtonDownFire2", enable)
        if enable then
            if not state.was_firing_wand then
                ComponentSetValue2(state.control_component, "mButtonFrameFire", GameGetFrameNum()+1)
            end
            ComponentSetValue2(state.control_component, "mButtonLastFrameFire", GameGetFrameNum())
        end
        throw = false
        state.was_firing_wand = enable
    end
end

local function init_state()
    local children = EntityGetAllChildren(ctx.my_player.entity)
    local items
    for _, child in pairs(children or {}) do
        if EntityGetName(child) == "inventory_quick" then
            items = child
        end
        local com = EntityGetFirstComponentIncludingDisabled(child, "GameEffectComponent")
        if com ~= nil then
            if ComponentGetValue2(com, "effect") == "CHARM" then
                EntityKill(child)
            end
        end
    end
    state = {
        entity = ctx.my_player.entity,
        control_component = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "ControlsComponent"),
        inv_component = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "InventoryGuiComponent"),
        data_component = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "CharacterDataComponent"),
        items = items,

        bad_potions = get_potions_of_type(bad_mats),
        good_potions = get_potions_of_type(good_mats),
        water_potions = get_potions_of_type(water_mats),

        had_potion = false,

        attack_wand = wandfinder.find_attack_wand({}),
        empty_wands = {},

        is_pheremoned = -120,
        aim_up = false,
        aim_down = false,

        was_firing_wand = false,
        was_a = false,
        was_w = false,
        was_d = false,
        init_timer = 0,

        control_a = false,
        control_w = false,
        control_d = false,

        dtype = 0
    }
    EntityAddComponent2(ctx.my_player.entity, "LuaComponent", {
        script_damage_received = "mods/quant.ew/files/system/notplayer_ai/damage_tracker.lua"
    })
    EntityAddComponent2(ctx.my_player.entity, "VariableStorageComponent", {
        _tags = "ew_damage_tracker",
        name = "ew_damage_tracker",
        value_int = 0,
    })
end

local target

local last_length

local last_did_hit = true

local function is_suitable_target(entity)
    return EntityGetIsAlive(entity) and not EntityHasTag(entity,"ew_notplayer")
end

local function choose_wand_actions()
    if (state.attack_wand ~= nil or bad_potion ~= nil) and target ~= nil and EntityGetIsAlive(target) then
        local t_x, t_y = EntityGetFirstHitboxCenter(target)
        if t_x == nil then
            t_x, t_y = EntityGetTransform(target)
        end
        if state.aim_up then
            t_y = t_y - 7
        elseif state.aim_down then
            t_y = t_y + 7
        end
        dont_throw = false
        aim_at(t_x, t_y)
        fire_wand(not last_did_hit and state.init_timer > 90 and not do_kick)-- or has_water_potion)
        return
    end
    fire_wand(false)
end

local stop_y = false

local swap_side = false

local give_space = 100

local on_right = false

local rest = false

local function choose_movement()
    if target == nil or (has_ambrosia(ctx.my_player.entity) and state.init_timer > 94) then
        state.control_a = false
        state.control_d = false
        state.control_w = false
        stop_y = false
        swap_side = false
        on_right = false
        return
    end
    local my_x, my_y = EntityGetTransform(ctx.my_player.entity)
    local t_x, t_y = EntityGetTransform(target)
    local dist = my_x - t_x
    local LIM = give_space
    if swap_side and (on_right ~= (my_x > t_x) or GameGetFrameNum() % 300 == 299) then
        swap_side = false
        give_space = 100
    end
    if swap_side then
        LIM = 0
        give_space = 100
    end
    if dist > 0 then
        state.control_a = dist > LIM
        state.control_d = not state.control_a
    else
        state.control_d = -dist > LIM
        state.control_a = not state.control_d
    end
    if (not stop_y) and ((last_did_hit and t_y < my_y + 80) or t_y < my_y) and (GameGetFrameNum() % 300) < 200 then
        state.control_w = true
        local did_hit, _, _ = RaytracePlatforms(my_x, my_y, my_x, my_y - 12)
        if did_hit then
            state.control_w = false
            stop_y = true
            local did_hit_down, _, _ = RaytracePlatforms(my_x, my_y, my_x, my_y + 12)
            if did_hit_down then
                if give_space == 100 then
                    give_space = math.abs(dist)
                else
                    give_space = give_space + 10
                end
                swap_side = false
            end
        end
    else
        if stop_y and (GameGetFrameNum() % 300) > 200 then
            stop_y = false
        end
        state.control_w = (GameGetFrameNum() % 60) > 45
    end

    if last_did_hit and t_y < my_y + 40 then
        local did_hit_1, _, _ = RaytracePlatforms(my_x, my_y, t_x, my_y)
        local did_hit_2, _, _ = RaytracePlatforms(t_x, my_y, t_x, t_y)
        if did_hit_1 and (not did_hit_2) then
            swap_side = true
            on_right = my_x > t_x
        end
    end

    local did_hit_1, _, _ = RaytracePlatforms(my_x, my_y, my_x + 16, my_y)
    local did_hit_2, _, _ = RaytracePlatforms(my_x, my_y, my_x - 16, my_y)
    if (dist > 0 and dist > give_space and did_hit_2) or (dist < 0 and -dist > give_space and did_hit_1) then
        if give_space == 100 then
            give_space = math.abs(dist)
        else
            give_space = give_space + 10
        end
        swap_side = false
    elseif give_space > 200 then
        local did_hit_3, _, _ = RaytracePlatforms(my_x, my_y, my_x + 100, my_y)
        local did_hit_4, _, _ = RaytracePlatforms(my_x, my_y, my_x - 100, my_y)
        if (dist > 0 and not did_hit_3) or (dist < 0 and not did_hit_4) then
            swap_side = true
        end
    end
    if (did_hit_1 and my_x > t_x) or (did_hit_2 and my_x < t_x) then
        swap_side = true
        on_right = my_x > t_x
    end

    if ComponentGetValue2(state.data_component, "mFlyingTimeLeft") < 0.2 and GameGetFrameNum() % 300 > 250 then
        rest = true
        give_space = give_space + 10
        swap_side = false
    end
    if rest and GameGetFrameNum() % 300 == 60 then
        rest = false
    end
    if rest or stop_y then
        state.control_w = false
    end
    if bathe then
        state.control_w = false
        state.control_a = false
        state.control_d = false
    end
    if GameGetFrameNum() % 20 == 0 and not last_did_hit then
        if give_space > 50 then
            give_space = give_space - 10
        else
            give_space = 100
        end
    end
    --GamePrint(state.dtype)
    if state.dtype == 32 then
        if (dist > 0 and did_hit_2) or (dist < 0 and did_hit_1) then
            give_space = give_space + 10
        else
            swap_side = true
        end
        state.control_w = false
    end
end

local function is_in_box(x1, x2, y1, y2, x, y)
    return x1 < x and x < x2 and y1 < y and y < y2
end

local function position_to_area_number(x, y)
    if np.GetGameModeNr() == 2 then
        if y < 1199 then
            return 1
        elseif y < 3759 then
            return 2
        elseif y < 6319 then
            return 3
        elseif y < 10415 then
            return 4
        elseif y < 12975 and (x < 2726 or x > 4135 or y < 12800) then
            return 5
        elseif is_in_box(5632, 7168, 14336, 15872, x, y) then
            return 10
        else
            return 6
        end
    elseif tonumber(SessionNumbersGetValue("NEW_GAME_PLUS_COUNT")) > 0 then
        if y < 1199 then
            return 1
        elseif y < 2735 then
            return 2
        elseif y < 6319 then
            return 3
        elseif y < 10415 then
            return 4
        elseif y < 12975 and (x < 2726 or x > 4135 or y < 12800) then
            return 5
        elseif is_in_box(5632, 7168, 14336, 15872, x, y) then
            return 10
        else
            return 6
        end
    else
        if y < 1199 then
            return 1
        elseif y < 2735 then
            return 2
        elseif y < 4783 then
            return 3
        elseif y < 6319 then
            return 4
        elseif y < 8367 then
            return 5
        elseif y < 10415 then
            return 6
        elseif y < 12975 and (x < 2726 or x > 4135 or y < 12800) then
            return 7
        elseif is_in_box(5632, 7168, 14336, 15872, x, y) then
            return 10
        else
            return 8
        end
    end
end

local function teleport_to_area(area)
    async(function()
        if np.GetGameModeNr() == 2 then
            if area == 1 then
                EntitySetTransform(ctx.my_player.entity, 191, 1514)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 191, 1514)
            elseif area == 2 then
                EntitySetTransform(ctx.my_player.entity, 191, 4066)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 191, 4066)
            elseif area == 3 then
                EntitySetTransform(ctx.my_player.entity, 191, 6626)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 191, 6626)
            elseif area == 4 then
                EntitySetTransform(ctx.my_player.entity, 191, 10722)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 191, 10722)
            elseif area == 5 then
                EntitySetTransform(ctx.my_player.entity, 3244, 13084)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 3244, 13084)
            elseif area == 9 then
                EntitySetTransform(ctx.my_player.entity, 6400, 15000)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 6400, 15000)
            end
        elseif tonumber(SessionNumbersGetValue("NEW_GAME_PLUS_COUNT")) > 0 then
            if area == 1 then
                EntitySetTransform(ctx.my_player.entity, 191, 1514)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 191, 1514)
            elseif area == 2 then
                EntitySetTransform(ctx.my_player.entity, 191, 3040)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 191, 3040)
            elseif area == 3 then
                EntitySetTransform(ctx.my_player.entity, 191, 6626)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 191, 6626)
            elseif area == 4 then
                EntitySetTransform(ctx.my_player.entity, 191, 10722)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 191, 10722)
            elseif area == 5 then
                EntitySetTransform(ctx.my_player.entity, 3244, 13084)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 3244, 13084)
            elseif area == 9 then
                EntitySetTransform(ctx.my_player.entity, 6400, 15000)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 6400, 15000)
            end
        else
            if area == 1 then
                EntitySetTransform(ctx.my_player.entity, 191, 1514)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 191, 1514)
            elseif area == 2 then
                EntitySetTransform(ctx.my_player.entity, 191, 3066)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 191, 3066)
            elseif area == 3 then
                EntitySetTransform(ctx.my_player.entity, 191, 5114)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 191, 5114)
            elseif area == 4 then
                EntitySetTransform(ctx.my_player.entity, 191, 6634)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 191, 6634)
            elseif area == 5 then
                EntitySetTransform(ctx.my_player.entity, 191, 8696)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 191, 8696)
            elseif area == 6 then
                EntitySetTransform(ctx.my_player.entity, 191, 10730)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 191, 10730)
            elseif area == 7 then
                EntitySetTransform(ctx.my_player.entity, 3244, 13084)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 3244, 13084)
            elseif area == 9 then
                EntitySetTransform(ctx.my_player.entity, 6400, 15000)
                wait(30)
                EntitySetTransform(ctx.my_player.entity, 6400, 15000)
            end
        end
    end)
end

local function teleport_to_next_hm()
    --BiomeMapGetName()
    --BIOME_MAP
    --MagicNumbersGetValue
    --ModIsEnabled("nightmare"), np.GetGameModeNr() == 3
    --tonumber(SessionNumbersGetValue("NEW_GAME_PLUS_COUNT")) > 0

    -- main x area -5646 < x < 5120
    -- main y area -1400 < y < 14336

    -- 1st area, y < 1104, exit  191,  1514
    -- 2nd area, y < 2640, exit  191,  3066
    -- 3rd area, y < 4688, exit  191,  5114
    -- 4th area, y < 6224, exit  191,  6634
    -- 5th area, y < 8272, exit  191,  8696
    -- 6th area, y < 10320, exit 191,  10730
    -- 7th area, y < 12880, exit 3244, 13084

    local my_area_num = -1
    local others_area_num = 100
    for peer_id, player_data in pairs(ctx.players) do
        local player = player_data.entity
        local x, y = EntityGetTransform(player)
        if x == nil or not (-5646 < x and x < 5120 and -1400 < y and y < 14336) and not is_in_box(5632, 7168, 14336, 15872, x, y) then
            return
        end
        if peer_id == ctx.my_id then
            my_area_num = position_to_area_number(x, y)
        elseif is_suitable_target(player) then
            local area_num = position_to_area_number(x, y)
            if area_num < others_area_num then
                others_area_num = area_num
            end
        end
    end
    if my_area_num < others_area_num then
        teleport_to_area(others_area_num - 1)
    end
end

local function teleport_outside_cursed()
    --17370 < x < 18470
    --53210 < x
    --14074, -820
    --35840
    --1100
    if np.GetGameModeNr() == 3 then
        return
    end
    local x, _ = EntityGetTransform(ctx.my_player.entity)
    local how_far_right = (x - (17370+550)) % 35840
    if how_far_right <= 550 or how_far_right >= (35840 - 550) then
        EntitySetTransform(ctx.my_player.entity, 14074, -820)
    end
end

local function hold_something()
    local ch_x, ch_y = EntityGetTransform(state.entity)
    if GameGetFrameNum() % 20 == 0 then
        find_new_wand()
    end
    local inventory = EntityGetFirstComponent(ctx.my_player.entity, "Inventory2Component")
    local holding = ComponentGetValue2(inventory, "mActualActiveItem")
    local i = 1
    local tablet = false
    if target ~= nil and EntityHasTag(target, "polymorphed") then
        for j, item in ipairs(state.bad_potions) do
            if EntityHasTag(item, "normal_tablet") then
                i = j
                tablet = true
                break
            end
        end
    end
    local can_not_tablet=false
    if not tablet then
        for j, item in ipairs(state.bad_potions) do
            if not EntityHasTag(item, "normal_tablet") then
                i = j
                can_not_tablet = true
                break
            end
        end
    end
    if bad_potion ~= nil and (holding == nil or holding ~= bad_potion) then
        table.remove(state.bad_potions, i)
        bad_potion = nil
        stop_potion = true
        do_kick = true
    end
    if good_potion ~= nil and (holding == nil or holding ~= good_potion) then
        table.remove(state.good_potions, 1)
        good_potion = nil
        stop_potion = true
        bathe = true
        do_kick = true
    end
    local douse = needs_douse(ctx.my_player.entity)
    local target_is_ambrosia = has_ambrosia(target) and not last_did_hit
    if state.water_potions[1] == nil or not is_potion_of_type(state.water_potions[1], water_mats) then
        table.remove(state.water_potions, 1)
        if water_potion ~= nil then
            water_potion = nil
            throw_water = false
            bathe = false
            do_kick = true
        end
    end
    if water_potion ~= nil and (((state.init_timer >= 90 and not last_did_hit) or not douse) or (holding == nil or holding ~= water_potion) or (throw_water and not target_is_ambrosia)) then
        water_potion = nil
        throw_water = false
        bathe = false
        do_kick = true
    end

    if GameGetFrameNum() % 120 == 40 then
        bathe = false
    end

    if GameGetFrameNum() % 120 == 60 then
        stop_potion = false
    end
    local ground_below, _, _ = RaytracePlatforms(ch_x, ch_y, ch_x, ch_y + 40)
    local is_ambrosia = has_ambrosia(ctx.my_player.entity)
    local has_water_potion = (not is_ambrosia or target_is_ambrosia) and #state.water_potions ~= 0 and (douse or target_is_ambrosia) and (state.init_timer < 90 or last_did_hit or target_is_ambrosia)
    local has_bad_potion = not has_water_potion and not is_ambrosia and #state.bad_potions ~= 0 and not last_did_hit and ((GameGetFrameNum() % 120 > 100 and state.init_timer > 120 and not stop_potion) or tablet)
    local has_good_potion = not has_water_potion and not is_ambrosia and #state.good_potions ~= 0 and not last_did_hit and GameGetFrameNum() % 120 < 20 and state.init_timer > 120 and not stop_potion and ground_below
    if GameGetFrameNum() % 10 == 0 and state.had_potion and #state.bad_potions == 0 and #state.good_potions == 0 then
        local has_a_potion = false
        for _, item in pairs(EntityGetAllChildren(state.items)) do
            if EntityHasTag(item, "potion") then
                has_a_potion = true
            end
        end
        state.had_potion = false
        if not has_a_potion then
            local pity_potion = EntityLoad("data/entities/items/pickup/potion_empty.xml", 0, 0)
            EntityAddChild(state.items, pity_potion)
            EntitySetComponentsWithTagEnabled(pity_potion, "enabled_in_world", false)
            EntitySetComponentsWithTagEnabled(pity_potion, "enabled_in_hand", false)
            EntitySetComponentsWithTagEnabled(pity_potion, "enabled_in_inventory", true)
        end
    end
    if has_water_potion or water_potion ~= nil then
        np.SetActiveHeldEntity(state.entity, state.water_potions[1], false, false)
        if water_potion == nil then
            water_potion = state.water_potions[1]
            do_kick = true
        end
        throw_water = target_is_ambrosia
        bathe = not target_is_ambrosia
    elseif (has_bad_potion or bad_potion ~= nil) and  (can_not_tablet or tablet) then
        if EntityHasTag(state.bad_potions[i], "potion") then
            state.had_potion = true
        end
        np.SetActiveHeldEntity(state.entity, state.bad_potions[i], false, false)
        if bad_potion == nil then
            bad_potion = state.bad_potions[i]
            do_kick = true
        end
    elseif has_good_potion or good_potion ~= nil then
        if EntityHasTag(state.bad_potions[i], "potion") then
            state.had_potion = true
        end
        np.SetActiveHeldEntity(state.entity, state.good_potions[1], false, false)
        if good_potion == nil then
            good_potion = state.good_potions[1]
            do_kick = true
        end
    else
        if state.attack_wand ~= nil then
            np.SetActiveHeldEntity(state.entity, state.attack_wand, false, false)
        end
    end
end

local target_has_ambrosia = false

local target_is_polied = false

local function better_player(length, did_hit, new_has_ambrosia, new_target_is_polied, potential_target)
    return (last_length == nil or
            (not did_hit and ((last_length > length or last_did_hit)
                    or (new_target_is_polied and not target_is_polied)
                    or (not new_has_ambrosia and target_has_ambrosia))
            ))
            and (not IsInvisible(potential_target) or length < 50*50)
end

local function find_target()
    local ch_x, ch_y = EntityGetTransform(state.entity)
    local potential_targets = EntityGetInRadiusWithTag(ch_x, ch_y, MAX_RADIUS, "ew_client") or {}
    local arm = EntityGetAllChildren(ctx.my_player.entity, "player_arm_r")[1]
    local x, y = EntityGetHotspot(arm, "hand", true)

    if target ~= nil then
        local x1, y1 = EntityGetTransform(target)
        if x1 == nil then
            target = nil
            last_length = nil
            last_did_hit = true
        else
            local dx = x - x1
            local dy = y - y1
            last_length = dx * dx + dy + dy
            if not is_suitable_target(target) or (last_length > 600 * 600) or (IsInvisible(target) and last_length > 50*50) then
                target = nil
                last_length = nil
                last_did_hit = true
            end
        end
    end
    if target ~= nil then
        local t_x, t_y = EntityGetFirstHitboxCenter(target)
        if t_x == nil then
            t_x, t_y = EntityGetTransform(target)
        end
        state.aim_up = false
        state.aim_down = false
        local did_hit, _, _ = RaytracePlatforms(x, y, t_x, t_y)
        if did_hit then
            did_hit, _, _ = RaytracePlatforms(x, y, t_x, t_y - 7)
            if not did_hit then
                state.aim_up = true
            else
                did_hit, _, _ = RaytracePlatforms(x, y, t_x, t_y + 7)
                if not did_hit then
                    state.aim_down = true
                end
            end
        end
        last_did_hit = did_hit
        target_has_ambrosia = has_ambrosia(target)
        target_is_polied = EntityHasTag(target, "polymorphed")
    end

    log("Trying to choose target")
    for _, potential_target in ipairs(potential_targets) do
        log("Trying "..potential_target)
        if is_suitable_target(potential_target) then
            local t_x, t_y = EntityGetFirstHitboxCenter(potential_target)
            if t_x == nil then
                t_x, t_y = EntityGetTransform(potential_target)
            end
            local dx = x - t_x
            local dy = y - t_y
            local length = dx * dx + dy * dy
            local did_hit, _, _ = RaytracePlatforms(x, y, t_x, t_y)
            local new_has_ambrosia = has_ambrosia(potential_target)
            local new_target_is_polied = EntityHasTag(target, "polymorphed")
            local success = false
            if better_player(length, did_hit, new_has_ambrosia, new_target_is_polied, potential_target) then
                success = true
                state.aim_up = false
                state.aim_down = false
            elseif did_hit then
                local did_hit_up, _, _ = RaytracePlatforms(x, y, t_x, t_y - 7)
                if better_player(length, did_hit_up, new_has_ambrosia, new_target_is_polied, potential_target) then
                    success = true
                    state.aim_up = true
                    state.aim_down = false
                elseif did_hit_up then
                    local did_hit_down, _, _ = RaytracePlatforms(x, y, t_x, t_y + 7)
                    if better_player(length, did_hit_down, new_has_ambrosia, new_target_is_polied, potential_target) then
                        success = true
                        target = potential_target
                        state.aim_up = false
                        state.aim_down = true
                    end
                end
            end
            if success then
                last_length = length
                last_did_hit = did_hit
                target_has_ambrosia = new_has_ambrosia
                target_is_polied = new_target_is_polied
                target = potential_target
            end
        end
    end
end

local kick_wait = 0

local function update()
    local var = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "VariableStorageComponent", "ew_damage_tracker")
    if GameGetFrameNum() % 30 == 0 then
        ComponentSetValue2(var, "value_int", 0)
    end
    state.dtype = ComponentGetValue2(var, "value_int")
    -- No taking control back, even after pressing esc.
    ComponentSetValue2(state.control_component, "enabled", false)
    ComponentSetValue2(state.inv_component, "mActive", false)
    if GameHasFlagRun("ending_game_completed") then
        return
    end

    state.init_timer = state.init_timer + 1

    find_target()

    do_kick = last_length ~= nil and last_length < 100

    hold_something()

    if state.is_pheremoned == -120 and has_pheremoned(ctx.my_player.entity) then
        state.is_pheremoned = GameGetFrameNum()
    end
    if do_kick and kick_wait + 16 < GameGetFrameNum() then
        kick_wait = GameGetFrameNum()
        ComponentSetValue2(state.control_component, "mButtonDownKick", true)
        ComponentSetValue2(state.control_component, "mButtonFrameKick", GameGetFrameNum()+1)
    else
        ComponentSetValue2(state.control_component, "mButtonDownKick", false)
    end
    choose_wand_actions()
    choose_movement()

    ComponentSetValue2(state.control_component, "mButtonDownLeft", state.control_a)
    if state.control_a and not state.was_a then
        ComponentSetValue2(state.control_component, "mButtonFrameLeft", GameGetFrameNum()+1)
    end
    state.was_a = state.control_a

    ComponentSetValue2(state.control_component, "mButtonDownRight", state.control_d)
    if state.control_d and not state.was_d then
        ComponentSetValue2(state.control_component, "mButtonFrameRight", GameGetFrameNum()+1)
    end
    state.was_d = state.control_d

    ComponentSetValue2(state.control_component, "mButtonDownDown", false)
    ComponentSetValue2(state.control_component, "mButtonDownUp", true)
    ComponentSetValue2(state.control_component, "mButtonDownFly", state.control_w)
    if state.control_w and not state.was_w then
        ComponentSetValue2(state.control_component, "mButtonFrameUp", GameGetFrameNum()+1)
        ComponentSetValue2(state.control_component, "mButtonFrameFly", GameGetFrameNum()+1)
    end
    state.was_w = state.control_w
    local _, y_n = EntityGetTransform(ctx.my_player.entity)
    ComponentSetValue2(state.control_component, "mFlyingTargetY", y_n - 10)

    if (GameGetFrameNum() % 60) == 59 then
        teleport_to_next_hm()
        teleport_outside_cursed()
    end
    EntityRemoveIngestionStatusEffect(ctx.my_player.entity, "CHARM")
end

function module.on_world_update()
    local active = GameHasFlagRun("ew_flag_notplayer_active")
    if active then
        if state == nil then
            init_state()
        end
        update()
    else
        state = nil
    end
end

return module