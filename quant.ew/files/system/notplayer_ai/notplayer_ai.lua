local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")

local state = nil
local module = {}

local function aim_at(x, y)
    
end


local function init_state()
    state = {
        entity = ctx.my_player.entity,
        control = EntityGetFirstComponentIncludingDisabled(ctx.my_player.entity, "ControlsComponent")
    }
end

local function choose_wand_actions()

end

local function choose_movement()

end

local function update()
    choose_wand_actions()
    choose_movement()
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