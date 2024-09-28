connect_steam = 使用steam联机
connect_steam_create = 创建联机大厅
connect_steam_connect = 连接到剪贴板中的大厅
connect_steam_workaround_label = 将大厅id粘贴到输入框进行连接：(如果你使用的是Wayland，则常规方式不起作用)
connect_steam_connect_2 = 在文本框中连接到大厅
connect_steam_connect_invalid_lobby_id = 剪切板内不包含大厅代码

connect_ip = 使用IP联机

lang_picker = 选择语言(language)

button_confirm = 确认
button_continue = 继续
button_retry = 重试
button_select_again = 再次选择
button_back = 返回

button_set_lang = 设置语言
button_open_discord = 打开Discord服务器

modman = Mod 管理
modman_found_automatically = 自动寻找路径：
modman_use_this = 使用这个路径
modman_select_manually = 手动选择路径
modman_path_to_exe = 选择noita.exe的路径
modman_invalid_path = 无效路径
modman_downloading = 正在下载Mod
modman_receiving_rel_info = 正在接收版本信息
modman_unpacking = 正在解压Mod
modman_installed = Mod已安装
modman_will_install_to = noita-proxy将会把Mod安装到:
modman_another_path = 选择其他路径

player_host = 房主
player_me = 我
player_player = 玩家

version_latest = (最新版)
version_check_failed = (无法检查更新)
version_checking = (检查更新中)
version_new_available = 有可用的更新，版本{ $new_version }

selfupdate = 自动更新
selfupdate_confirm = 确认更新
selfupdate_receiving_rel_info = 接收版本信息...
selfupdate_updated = noita-proxy已更新！立即重启。
selfupdate_unpacking = 正在解压中...

noita_not_yet = 还没准备好，请等待Noita启动。
noita_can_connect = 正在等待Noita连接。是时候开一把新的Noita了！
noita_connected = 本地Noita实例已连接。

netman_save_lobby = 将大厅id保存到剪贴板
netman_show_settings = 显示设置窗口
netman_apply_settings = 应用设置并在下一次启动时生效

ip_note = 注意：使用steam联机更可靠，如果可以的话请使用steam联机
ip_connect = 连接至IP
ip_host = 创建一个服务器

error_occured = 发生错误：
error_missing_version_field = 大厅没有版本字段。大厅是由旧的noita-proxy版本创建的。
error_version_mismatch = 大厅是由具有不同版本的noita-proxy版本创建的：{$remote_version}。当前已安装的版本是{$current_version}。
error_lobby_does_not_exist = 大厅不存在

launcher_already_started = Noita已启动。
launcher_no_command = 无法启动Noita：没有启动命令。
launcher_no_command_2 = 你可以使用 --launch-cmd <command> 选项指定启动命令。
launcher_no_command_3 = 你可以在steam的启动项中放入`noita-proxy --launch-cmd "%command%" 用于启动游戏时执行其他命令。
launcher_start_game = 启动Noita
launcher_only_when_awaiting = 只能在“等待Noita连接”状态下启动游戏。

connect_settings = 游戏设置
connect_settings_debug = 调试设置
connect_settings_debug_en = 调试/作弊模式
connect_settings_debug_fixed_seed = 使用固定种子
connect_settings_seed = 种子：
connect_settings_wsv = 要使用的世界同步版本：
connect_settings_player_tether = 启用玩家系绳
connect_settings_player_tether_desc = 玩家系绳：如果其他玩家距离主机太远，则将其传送到主机。
connect_settings_player_tether_length = 系绳长度
connect_settings_item_dedup = 消除由世界生成的重复(同步)项
connect_settings_enemy_hp_scale = 敌人血量比例
connect_settings_local = 本地设置
connect_settings_autostart = 自动启动游戏

## Game settings

Enable-friendly-fire = Enable friendly fire
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
world-sync-is-pixel-sync-note = 注意：世界同步是指同步世界像素(材质)的部分。敌人和其他实体不受此影响。
Higher-values-result-in-less-performance-impact = 该值越大，对性能影响越小。
World-will-be-synced-every-this-many-frames = 该值将作为世界同步的间隔(帧)。

## Savestate

New-game = 新的游戏
Continue = 继续游戏
savestate_desc = 检测到上一次的存档。你想要继续启动该存档还是开启一局新游戏(并重置存档)？
An-in-progress-run-has-been-detected = 检查到正在运行的存档。

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

Show-debug-info = Show debug info
hint_spectate = Use [',' or d-pad-left] and ['.' or d-pad-right] keys to spectate over other players.
hint_ping = [Middle mouse button or right thumb stick] spawns a ping
