local ctx = dofile_once("mods/quant.ew/files/core/ctx.lua")
local net = dofile_once("mods/quant.ew/files/core/net.lua")
local np = require("noitapatcher")

local rpc = net.new_rpc_namespace()

local module = {}

ModLuaFileAppend("data/scripts/magic/fungal_shift.lua", "mods/quant.ew/files/system/fungal_shift/append/fungal_shift.lua")

local log_messages = {
    "$log_reality_mutation_00",
    "$log_reality_mutation_01",
    "$log_reality_mutation_02",
    "$log_reality_mutation_03",
    "$log_reality_mutation_04",
    "$log_reality_mutation_05",
}

rpc.opts_reliable()
function rpc.fungal_shift(conversions, iter, from_material_name)
    dofile_once("data/scripts/lib/utilities.lua")

    local entity = ctx.my_player.entity
    local x, y = EntityGetTransform(entity)

    GlobalsSetValue("fungal_shift_iteration", iter)
    for _, conv in ipairs(conversions) do
        ConvertMaterialEverywhere(conv[1], conv[2])
        GameCreateParticle( CellFactory_GetName(conv[1]), x-10, y-10, 20, rand(-100,100), rand(-100,-30), true, true )
        GameCreateParticle( CellFactory_GetName(conv[1]), x+10, y-10, 20, rand(-100,100), rand(-100,-30), true, true )
    end

    -- remove tripping effect
    EntityRemoveIngestionStatusEffect( entity, "TRIP" );

    -- audio
    GameTriggerMusicFadeOutAndDequeueAll( 5.0 )
    GameTriggerMusicEvent( "music/oneshot/tripping_balls_01", false, x, y )

    -- particle fx
    local eye = EntityLoad( "data/entities/particles/treble_eye.xml", x,y-10 )
    if eye ~= 0 then
        EntityAddChild( entity, eye )
    end

    -- log
    local log_msg = ""
    if from_material_name ~= "" then
        log_msg = GameTextGet( "$logdesc_reality_mutation", from_material_name )
        GamePrint( log_msg )
    end
    GamePrintImportant( random_from_array( log_messages ), log_msg, "data/ui_gfx/decorations/3piece_fungal_shift.png" )
    local frame = GameGetFrameNum()
    GlobalsSetValue( "fungal_shift_last_frame", tostring(frame) )

    -- add ui icon
    local add_icon = true
    local children = EntityGetAllChildren(entity)
    if children ~= nil then
        for _,it in ipairs(children) do
            if ( EntityGetName(it) == "fungal_shift_ui_icon" ) then
                add_icon = false
                break
            end
        end
    end

    if add_icon then
        local icon_entity = EntityCreateNew( "fungal_shift_ui_icon" )
        EntityAddComponent( icon_entity, "UIIconComponent",
        {
            name = "$status_reality_mutation",
            description = "$statusdesc_reality_mutation",
            icon_sprite_file = "data/ui_gfx/status_indicators/fungal_shift.png"
        })
        EntityAddChild( entity, icon_entity )
    end
end

local conversions = {}

util.add_cross_call("ew_fungal_shift_conversion", function(from_mat, to_mat)
    table.insert(conversions, {from_mat, to_mat})
end)

util.add_cross_call("ew_fungal_shift", function(iter, from_material_name)
    rpc.fungal_shift(conversions, iter, from_material_name)
    conversions = {}
end)

local last_fungals

function rpc.give_fungals(changed_materials)
    local world = GameGetWorldStateEntity()
    local com = EntityGetFirstComponentIncludingDisabled(world, "WorldStateComponent")
    local my_changed_materials = ComponentGetValue2(com, "changed_materials")
    local n = #my_changed_materials
    if n == nil then
        n = 0
    end
    if #changed_materials > n then
        if last_fungals ~= nil and #last_fungals == #changed_materials then
            for i = n + 1, #changed_materials, 2 do
                ConvertMaterialEverywhere(changed_materials[i], changed_materials[i + 1])
            end
            last_fungals = nil
        else
            last_fungals = changed_materials
        end
    end
end

function module.on_world_update()
    if ctx.my_id == ctx.host_id and GameGetFrameNum() % 600 == 34 then
        local world = GameGetWorldStateEntity()
        local com = EntityGetFirstComponentIncludingDisabled(world, "WorldStateComponent")
        local changed_materials = ComponentGetValue2(com, "changed_materials")
        if changed_materials ~= nil then
            local mats = {}
            for _, mat in ipairs(changed_materials) do
                table.insert(mats, CellFactory_GetType(mat))
            end
            rpc.give_fungals(mats)
        end
    end
end

return module