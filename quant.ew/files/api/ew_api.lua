local ew_api = {}

-- Creates an rpc namespace. See ew_api_example for how to use it.
ew_api.new_rpc_namespace = net.new_rpc_namespace_with_id

-- In rpc, returns player_data of a peer who called that rpc.
function ew_api.rpc_player_data()
    return ctx.rpc_player_data
end

return ew_api
