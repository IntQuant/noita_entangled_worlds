# List of available hooks:

 - `ctx.hook.on_world_initialized()` - called on OnWorldInitialized.
 - `ctx.hook.on_new_entity(ent)` - called on new alive entity.
 - `ctx.hook.on_world_update()` - called on OnWorldPreUpdate.
 - `ctx.hook.on_world_update_post()` - called on OnWorldPostUpdate.
 - `ctx.hook.on_world_update_client()` - called on OnWorldPreUpdate, but only on clients.
 - `ctx.hook.on_world_update_host()` - called on OnWorldPreUpdate, but only on host.
 - `ctx.hook.on_new_player_seen(new_playerdata, player_count)` - called the first time player with this peer_id has entered the world.
 - `ctx.hook.on_local_player_spawn(my_player)` - called when the *local* gets spawned. `my_player` is playerdata table for this player.
 - `ctx.hook.on_client_spawned(peer_id, new_playerdata)`
 - `ctx.hook.on_should_send_updates()` - called either when we connect, or somebody else connects (and sends "welcome" message).
 - `ctx.hook.on_draw_debug_window(imgui)`
 - `ctx.hook.on_local_player_polymorphed(currently_polymorphed)`
 - `ctx.hook.on_client_polymorphed(peer_id, player_data)`
 - `ctx.hook.on_late_init()` - called on OnModPostInit.