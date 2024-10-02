function get_players()
	return EntityGetWithTag("ew_peer")
end


function _streaming_run_event(id)
    CrossCall("ew_run_streaming_event", id)
end
