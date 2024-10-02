ModLuaFileAppend("data/scripts/streaming_integration/event_utilities.lua", "mods/quant.ew/files/system/streaming_sync/event_hook.lua")

dofile_once("data/scripts/streaming_integration/event_list.lua")

local rpc = net.new_rpc_namespace()

local module = {}

rpc.opts_reliable()
rpc.opts_everywhere()
function rpc.remote_run_event(id)
    for i,evt in ipairs(streaming_events) do
		if evt.id == id then
			if evt.action_delayed ~= nil then
				if evt.delay_timer ~= nil then
                    local p = ctx.my_player.entity
                    for a,b in ipairs( p ) do
                        add_timer_above_head( b, evt.id, evt.delay_timer )
                    end
				end
			elseif evt.action ~= nil then
				evt.action(evt)
			end
            if event_weights ~= nil then
                event_weights[i] = -1.0
            end
            GamePrint("Incoming event "..GameTextGetTranslatedOrNot(evt.ui_name).." from "..ctx.rpc_player_data.name)
			break
		end
	end
end

np.CrossCallAdd("ew_run_streaming_event", rpc.remote_run_event)

return module