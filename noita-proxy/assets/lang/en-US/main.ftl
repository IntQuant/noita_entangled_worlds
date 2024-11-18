connect_steam = Connect using steam
connect_steam_create = Create lobby
connect_steam_connect = Connect to lobby in clipboard
connect_steam_workaround_label = Connect by pasting lobby id in this field: (In case you are using Wayland and the normal way doesn't work)
connect_steam_connect_2 = Connect to lobby in the text field
connect_steam_connect_invalid_lobby_id = Clipboard does not contain a lobby code

connect_ip = Connect using ip

lang_picker = Choose a language

button_confirm = Confirm
button_continue = Continue
button_retry = Retry
button_select_again = Select again
button_back = Back

button_set_lang = Select language
button_open_discord = Discord server

modman = Mod manager
modman_found_automatically = Found a path automatically:
modman_use_this = Use this one
modman_select_manually = Select manually
modman_path_to_exe = Select path to noita.exe
modman_invalid_path = This path is not valid
modman_downloading = Downloading mod...
modman_receiving_rel_info = Receiving release info...
modman_unpacking = Unpacking mod...
modman_installed = Mod has been installed!
modman_will_install_to = Proxy will install the mod to:
modman_another_path = Select a different path

player_host = Host
player_me = Me
player_player = Player

version_latest = (latest)
version_check_failed = (could not check for updates)
version_checking = (checking for updates)
version_new_available = Update available to { $new_version }

selfupdate = Self update
selfupdate_confirm = Confirm update
selfupdate_receiving_rel_info = Receiving release info...
selfupdate_updated = Proxy updated! Restart it now.
selfupdate_unpacking = Unpacking...

noita_not_yet = Not yet ready. Please wait before starting noita.
noita_can_connect = Awaiting Noita connection. It's time to start new game in Noita now!
noita_connected = Local Noita instance connected.

netman_save_lobby = Save lobby id to clipboard
netman_show_settings = Show settings screen
netman_apply_settings = Apply settings to be used in the next run
apply_default_settings = Resets settings to default

ip_note = Note: steam networking is more reliable. Use it, if possible.
ip_connect = Connect to IP
ip_host = Create a server

error_occured = An error occured:
error_missing_version_field = Lobby does not have a version field. The lobby was created by an old proxy version.
error_version_mismatch = Lobby was created by proxy with a different version: { $remote_version }. You have { $current_version } currently installed.
error_lobby_does_not_exist = Lobby does not exist.

launcher_already_started = Noita is already started.
launcher_no_command = Can't start noita: no launch command.
launcher_no_command_2 = Launch command can be specified with --launch-cmd <command> option.
launcher_no_command_3 = You can put `noita-proxy --launch-cmd "%command%"` in steam's launch options to intercept whatever command steam uses to start the game.
launcher_start_game = Start noita
launcher_end_run = End run
launcher_end_run_confirm = Confirm
launcher_only_when_awaiting = Can only launch the game in "waiting for noita connection" state.

connect_settings = Game settings
connect_settings_debug = Debug settings
connect_settings_debug_en = Debug/cheat mode
connect_settings_debug_fixed_seed = Use fixed seed
connect_settings_seed = Seed:
connect_settings_max_players = Max players
connect_settings_wsv = World sync version to use:
connect_settings_player_tether = Player tether enabled
connect_settings_player_tether_desc = Player tether: Teleports clients to host if they get far enough.
connect_settings_player_tether_length = Tether length
connect_settings_item_dedup = Deduplicate (sync) items spawned by world generation.
connect_settings_enemy_hp_scale = Enemy hp scale.
connect_settings_local = Local settings
connect_settings_autostart = Start the game automatically

## Game settings

connect_settings_spacewars = Allow using steam networking even if you don't have the game on steam, in case you have the gog version of the game. All players need this ticked to work, restart proxy to take effect
Health-per-player = Health per player
Enable-friendly-fire = Enable friendly fire, allows picking teams in lobby
Have-perk-pools-be-independent-of-each-other = Have perk pools be independent of each other
Amount-of-chunks-host-has-loaded-at-once-synced-enemies-and-physics-objects-need-to-be-loaded-in-by-host-to-be-rendered-by-clients = Amount of chunks host has loaded at once, synced enemies and physics objects need to be loaded in by host to be rendered by clients
local_health_desc_1 = Every player has their own health, run ends when all player are dead.
local_health_desc_2 = There is a respawn mechanic.
Health-percent-lost-on-reviving = Percent of Max HP lost on reviving
global_hp_loss = Lose HP globally
no_material_damage = No material damage
perma_death = Perma death
physics_damage = Physics damage
shared_health_desc_1 = Health is shared, but scales with player count.
shared_health_desc_2 = Percentage-based damage and full heals are adjusted.
shared_health_desc_3 = The original mode.
Local-health = Local health
Shared-health = Shared health
Game-mode = Game mode
world-sync-is-pixel-sync-note = Note: World sync refers to the part that syncs pixels(materials) of the world. Enemies and other entities aren't affected by this.
Higher-values-result-in-less-performance-impact = Higher values result in less performance impact.
World-will-be-synced-every-this-many-frames = World will be synced every this many frames.

## Savestate

New-game = New game
Continue = Continue
savestate_desc = Savestate from a previous run has been detected. Do you wish to continue that run, or to start a new game (and reset the savestate)?
An-in-progress-run-has-been-detected = An in-progress run has been detected.

## Player appearance

Gem = Gem
Amulet = Amulet
Crown = Crown
Cape-edge-color = Cape edge color
Cape-color = Cape color
Forearm-color = Forearm color
Arm-color = Arm color
Alt-color = Alt color
Main-color = Main color
Reset-colors-to-default = Reset colors to default
Shift-hue = Shift hue

## Connected

Show-debug-info = Show debug/connection info
hint_spectate = Use [',' or d-pad-left] and ['.' or d-pad-right] keys to spectate over other players. '/' for self
hint_ping = [Middle mouse button or right thumb stick] spawns a ping
Show-debug-plot = Show debug plot
Record-everything-sent-to-noita = Record EVERYTHING sent to noita.

## IP Connect

ip_could_not_connect = Could not connect
ip_wait_for_connection = Connecting to ip...
## Info

info_stress_tests = We're doing public lobbies (a.k.a stress tests) every saturday, 18:00 UTC. Join our discord for more info.
Info = Info