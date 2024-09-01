# List of available hooks:

 - `ctx.hook.on_world_initialized()`
 - `ctx.hook.on_world_update()`
 - `ctx.hook.on_world_update_client()`
 - `ctx.hook.on_world_update_host()`
 - `ctx.hook.on_new_player_seen(new_playerdata, player_count)` - called the first time player with this peer_id has entered the world
 - `ctx.hook.on_local_player_spawn(my_player)`
 - `ctx.hook.on_client_spawned(peer_id, new_playerdata)`
 - `ctx.hook.on_should_send_updates()` - called either when we connect, or somebody else connects (and sends "welcome" message)
 - `ctx.hook.on_draw_debug_window(imgui)`
 - `ctx.hook.on_local_player_polymorphed(currently_polymorphed)`
 - `ctx.hook.on_client_polymorphed(peer_id, player_data)`
