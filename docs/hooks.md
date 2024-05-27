# List of available hooks:

 - `ctx.hook.on_world_update()`
 - `ctx.hook.on_world_update_client()`
 - `ctx.hook.on_world_update_host()`
 - `ctx.hook.on_local_player_spawn(my_player)`
 - `ctx.hook.on_client_spawned(peer_id, new_playerdata)`
 - `ctx.hook.on_should_send_updates()` - called either when we connect, or somebody else connects (and sends "welcome" message)
