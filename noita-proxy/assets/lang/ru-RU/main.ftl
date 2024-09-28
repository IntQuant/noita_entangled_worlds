connect_steam = Подключение по Steam
connect_steam_create = Создать лобби
connect_steam_connect = Подключиться к лобби в буфере обмена
connect_steam_workaround_label = Подключитесь вставив код лобби в это поле: (на случай, если вы используете Wayland и обычный способ не работает)
connect_steam_connect_2 = Подключитесь к лобби в текстовом поле
connect_steam_connect_invalid_lobby_id = Буфер обмена не содержит кода лобби.

connect_ip = Подключение по IP

lang_picker = Выберите язык

button_confirm = Подтвердить
button_continue = Продолжить
button_retry = Попробовать снова
button_select_again = Выбрать ещё раз
button_back = Назад

button_set_lang = Выбрать язык
button_open_discord = Discord сервер

modman = Установка мода
modman_found_automatically = Автоматически найденный путь:
modman_use_this = Использовать этот
modman_select_manually = Выбрать вручную
modman_path_to_exe = Выберите путь к noita.exe
modman_invalid_path = Этот путь не подходит
modman_downloading = Скачивание мода...
modman_receiving_rel_info = Получение информации о релизе...
modman_unpacking = Распаковка мода...
modman_installed = Мод установлен!
modman_will_install_to = Мод будет установлен в:
modman_another_path = Выбрать другой путь

player_host = Хост
player_me = Я
player_player = Игрок

version_latest = (последняя)
version_check_failed = (не удалось проверить обновления)
version_checking = (проверка обновлений)
version_new_available = Доступно обновление до { $new_version }

selfupdate = Автообновления
selfupdate_confirm = Подтвердить обновление
selfupdate_receiving_rel_info = Получение информации о релизе...
selfupdate_updated = Прокси был обновлён! Теперь его можно перезапустить.
selfupdate_unpacking = Распаковка...

noita_not_yet = Ещё не готово. Подождите, прежде чем запускать игру.
noita_can_connect = Ожидается подключение из игры. Можно запускать игру.
noita_connected = Локальный инстанс Ноиты подключен.

netman_save_lobby = Сохранить код лобби в буфер обмена
netman_show_settings = Отобразить экран настроек
netman_apply_settings = Применить настройки для использования в следующем забеге

ip_note = Рекомендуется использовать подключение через Steam, поскольку оно работает стабильнее.
ip_connect = Подключиться по IP
ip_host = Создать сервер

error_occured = Произошла ошибка:
error_missing_version_field = Лобби было создано очень старой версией прокси.
error_version_mismatch = Версии прокси не совпадают. Версия хоста: { $remote_version }. Установленная сейчас версия: { $current_version }.
error_lobby_does_not_exist = Лобби не существует.

launcher_already_started = Noita уже запущена.
launcher_no_command = Не получается запустить Noita: отсутствует команда запуска.
launcher_no_command_2 = Launch command can be specified with --launch-cmd <command> option.
launcher_no_command_3 = You can put `noita-proxy --launch-cmd "%command%"` in steam's launch options to intercept whatever command steam uses to start the game.
launcher_start_game = Запустить Noita
launcher_only_when_awaiting = Can only launch the game in "waiting for noita connection" state.

connect_settings = Настройки игры
connect_settings_debug = Настройки разработчика
connect_settings_debug_en = Включить читы
connect_settings_debug_fixed_seed = Фиксированный сид мира
connect_settings_seed = Сид:
connect_settings_wsv = Версия синхронизатора мира:
connect_settings_player_tether = Предел расстояния до хоста
connect_settings_player_tether_desc = Телепортирует игроков к хосту, если они слишком далеко.
connect_settings_player_tether_length = Максимальная длина
connect_settings_item_dedup = Синхронизация элементов, созданных при генерации мира.
connect_settings_enemy_hp_scale = Модификатор здоровья противников.
connect_settings_local = Локальные настройки
connect_settings_autostart = Запускать игру автоматически

## Game settings

Enable-friendly-fire = Включить дружественный огонь
Have-perk-pools-be-independent-of-each-other = Сделать перки локальными для каждого игрока
Amount-of-chunks-host-has-loaded-at-once-synced-enemies-and-physics-objects-need-to-be-loaded-in-by-host-to-be-rendered-by-clients = Количество чанков, загруженных хостом за один раз, враги и физические объекты должны быть загружены хостом для передачи другим игрокам
local_health_desc_1 = У каждого игрока свое здоровье, забег заканчивается, когда все игроки умрут.
local_health_desc_2 = Включена механика возрождения.
shared_health_desc_1 = Здоровье общее, но скалируется в зависимости от количества игроков.
shared_health_desc_2 = Процентный урон и полное исцеление скорректированы.
shared_health_desc_3 = Оригинальный игровой режим.
Local-health = Локальное здоровье
Shared-health = Общее здоровье
Game-mode = Игровой режим
world-sync-is-pixel-sync-note = Примечание: Синхронизация мира относится к части, которая синхронизирует пиксели мира. Враги и другие сущности не затронуты этим.
Higher-values-result-in-less-performance-impact = Чем выше значения, тем меньше влияние на производительность
World-will-be-synced-every-this-many-frames = Мир будет синхронизироваться каждый раз через указанное количество кадров

## Savestate

New-game = Начать новую
Continue = Продолжить игру
savestate_desc = Хотите продолжить игру или начать новую (и сбросить сохранение)?
An-in-progress-run-has-been-detected = Была обнаружена незавершённая игра

## Player appearance

Gem = Самоцвет
Amulet = Амулет
Crown = Корона
Cape-edge-color = Цвет кромки плаща
Cape-color = Цвет плаща
Forearm-color = Цвет левой руки
Arm-color = Цвет правой руки
Alt-color = Дополнительный цвет
Main-color = Главный цвет
Reset-colors-to-default = Сбросить цвета
Shift-hue = Смещение тона

## Connected

Show-debug-info = Показать отладочную информацию
hint_spectate = Используйте [',' или левая сегмент d-pad] и ['.' или правый сегмент d-pad] для наблюдения за другими игроками.
hint_ping = [Средняя кнопка мыши или правый стик] создают пинг

## IP Connect

ip_could_not_connect = Could not connect
ip_wait_for_connection = Connecting to ip...
