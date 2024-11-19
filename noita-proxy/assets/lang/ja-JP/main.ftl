connect_steam = Steamで接続する
connect_steam_create = ロビーを作成する
connect_steam_connect = クリップボードのロビーIDでロビーに接続する
connect_steam_workaround_label = フィールドにロビーIDを貼り付けて接続する: (Waylandを使用していて通常の方法が機能しない場合)
connect_steam_connect_2 = テキストフィールドでロビーに接続
connect_steam_connect_invalid_lobby_id = クリップボードにロビーIDが含まれていません

connect_ip = IP接続する

lang_picker = 言語を選択する

button_confirm = 確認
button_continue = 続行
button_retry = リトライ
button_select_again = 再選択
button_back = 戻る

button_set_lang = 言語を選択
button_open_discord = Discordサーバー

modman = MOD マネージャー
modman_found_automatically = 見つかった場所:
modman_use_this = これを利用する
modman_select_manually = 手動で選択する
modman_path_to_exe = noita.exe の場所を選択する
modman_invalid_path = このパスは無効です
modman_downloading = Modをダウンロード中...
modman_receiving_rel_info = リリース情報を受信中...
modman_unpacking = Modを解凍中...
modman_installed = Modがインストールされました!
modman_will_install_to = ProxyがModをインストールする場所:
modman_another_path = 別の場所を選択する

player_host = ホスト
player_me = 自分
player_player = プレイヤー

version_latest = (最新)
version_check_failed = (更新を確認できませんでした)
version_checking = (更新を確認中)
version_new_available = バージョン { $new_version } に更新可能です

selfupdate = 自動更新
selfupdate_confirm = 更新を確認
selfupdate_receiving_rel_info = リリース情報を受信中...
selfupdate_updated = Proxyは更新されました! 今すぐ再起動してください.
selfupdate_unpacking = 解凍中...

noita_not_yet = まだ準備中です。Noitaの開始をお待ち下さい。
noita_can_connect = Noitaに接続中です。今すぐNoitaで「新規ゲーム」を開始してください！
noita_connected = Noitaに接続されました。

netman_save_lobby = ロビーIDをクリップボードに保存する
netman_show_settings = 設定画面を表示する
netman_apply_settings = 次のゲームで適応される設定を適用する

ip_note = 注意: Steamネットワーキングの方が信頼性があります。可能であればSteamネットワーキング使用してください。
ip_connect = IP接続する
ip_host = サーバーを作成する

error_occured = エラーが発生しました:
error_missing_version_field = ロビーにversion fieldがありません。このロビーは古いバージョンによって作成されました。
error_version_mismatch = ロビーは異なるバージョンのプロキシによって作成されました: { $remote_version }。現在インストールされているバージョンは{ $current_version }です。
error_lobby_does_not_exist = ロビーが存在しません。

launcher_already_started = Noitaはすでに開始されています。
launcher_no_command = Noitaを開始できません: 起動コマンドがありません。
launcher_no_command_2 = 起動コマンドは --launch-cmd <command> オプションで指定できます。
launcher_no_command_3 = Steamの起動オプションに `noita-proxy --launch-cmd "%command%"` を入力すると、Steamがゲームを開始する際のコマンドを確認できます。
launcher_start_game = Noitaを開始する
launcher_only_when_awaiting = 「Noita接続待機中」状態のときのみゲームを開始できます。

connect_settings = ゲーム設定
connect_settings_debug = デバッグ設定
connect_settings_debug_en = デバッグ/チートモード
connect_settings_debug_fixed_seed = 固定Seedを利用する
connect_settings_seed = Seed:
connect_settings_wsv = World syncに利用する同期バージョン:
connect_settings_player_tether = プレイヤーテザーを有効にする
connect_settings_player_tether_desc = プレイヤーテザーとは: 参加者がホストから一定の範囲以上離れた場合にてレポートする
connect_settings_player_tether_length = テザーの長さ
connect_settings_item_dedup = ワールド生成の同期で重複したアイテムを削除する
connect_settings_enemy_hp_scale = 敵HPのスケーリング
connect_settings_local = ローカル設定
connect_settings_autostart = ゲームを自動的に開始する

## Game settings

connect_settings_spacewars = Allow using steam networking even if you don't have the game on steam, in case you have the gog version of the game. All players need this ticked to work, restart proxy to take effect
Health-per-player = Health per player
Enable-friendly-fire = Enable friendly fire, allows picking teams in lobby
Have-perk-pools-be-independent-of-each-other = Have perk pools be independent of each other
Amount-of-chunks-host-has-loaded-at-once-synced-enemies-and-physics-objects-need-to-be-loaded-in-by-host-to-be-rendered-by-clients = Amount of chunks host has loaded at once, synced enemies and physics objects need to be loaded in by host to be rendered by clients
local_health_desc_2 = There is a respawn mechanic.
local_health_desc_1 = Every player has their own health, run ends when all player are dead.
shared_health_desc_3 = The original mode.
shared_health_desc_2 = Percentage-based damage and full heals are adjusted.
shared_health_desc_1 = Health is shared, but scales with player count.
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
hint_spectate = Use [',' or d-pad-left] and ['.' or d-pad-right] keys to spectate over other players.
hint_ping = [Middle mouse button or right thumb stick] spawns a ping

## IP Connect

ip_could_not_connect = Could not connect
ip_wait_for_connection = Connecting to ip...
## Info

info_stress_tests = We're doing public lobbies (a.k.a stress tests) every saturday, 18:00 UTC. Join our discord for more info.
Info = Info
## Local settings

connect_settings_random_ports = Don't use a predetermined port. Makes things a bit more robust and allows multiple proxies to be launched on the same computer, but Noita will have to be launched through the proxy.