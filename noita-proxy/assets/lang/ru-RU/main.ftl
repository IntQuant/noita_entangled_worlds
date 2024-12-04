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
noita_can_connect = Ожидается подключение. Можно запускать игру.
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
launcher_end_run = Закончить забег
launcher_end_run_confirm = Подтвердить
launcher_only_when_awaiting = Запустить игру можно только в состоянии «Ожидается подключение»

connect_settings = Настройки игры
connect_settings_debug = Настройки разработчика
connect_settings_debug_en = Включить читы
connect_settings_debug_fixed_seed = Фиксированный сид мира
connect_settings_seed = Сид:
connect_settings_max_players = Максимум игроков
connect_settings_wsv = Версия синхронизатора мира:
connect_settings_player_tether = Предел расстояния до хоста
connect_settings_player_tether_desc = Телепортирует игроков к хосту, если они слишком далеко.
connect_settings_player_tether_length = Максимальная длина
connect_settings_item_dedup = Синхронизация предметов, созданных при генерации мира.
connect_settings_enemy_hp_scale = Модификатор здоровья противников.
connect_settings_local = Локальные настройки
connect_settings_autostart = Запускать игру автоматически

## Game settings

connect_settings_spacewars = Allow using steam networking even if you don't have the game on steam, in case you have the gog version of the game. All players need this ticked to work, restart proxy to take effect
Health-per-player = Стартовое здоровье
Enable-friendly-fire = Включить дружественный огонь
Have-perk-pools-be-independent-of-each-other = Сделать перки локальными для каждого игрока
Amount-of-chunks-host-has-loaded-at-once-synced-enemies-and-physics-objects-need-to-be-loaded-in-by-host-to-be-rendered-by-clients = Количество чанков, загруженных хостом за один раз, враги и физические объекты должны быть загружены хостом для передачи другим игрокам
local_health_desc_1 = У каждого игрока свое здоровье, забег заканчивается, когда все игроки умрут.
local_health_desc_2 = Включена механика возрождения.
Health-percent-lost-on-reviving = Процент потери здоровья при возрождении
global_hp_loss = Все игроки теряют здоровье
no_material_damage = Отключить урон от физики
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

Shift-hue = Смещение тона
Main-color = Первичный
Alt-color = Вторичный
Arm-color = Правая рука
Forearm-color = Левая руки
Cape-color = Плащ
Cape-edge-color = Кромка плаща
Gem = Самоцвет
Amulet = Амулет
Crown = Корона
Reset-colors-to-default = Сбросить цвета

## Connected

Show-debug-info = Показать отладочную информацию
hint_ping = [Средняя кнопка мыши или правый стик] создают метку
hint_spectate = Используйте [',' или левый сегмент d-pad] и ['.' или правый сегмент d-pad] для наблюдения за другими игроками.
Show-debug-plot = Показать отладочный график
Record-everything-sent-to-noita = Записывать всё что отправляется в игру

## IP Connect

ip_could_not_connect = Не удалось подключиться
ip_wait_for_connection = Подключение к ip...

## Info

info_stress_tests = We're doing public lobbies (a.k.a stress tests) every saturday, 18:00 UTC. Join our discord for more info.
Info = Info
## Local settings

connect_settings_random_ports = Don't use a predetermined port. Makes things a bit more robust and allows multiple proxies to be launched on the same computer, but Noita will have to be launched through the proxy.

## UX settings

ping-note = Параметры стрелочки-пинга
ping-lifetime = Время жизни стрелки в секундах.
ping-scale = Размер стрелки в юнитах.
ping-lifetime-tooltip = Этот параметр изменяет время жизни стрелочки (секунды*60, т.к. игра должна работать в 60 фпс?). Диапазон: 0-60 секунд.
ping-scale-tooltip = Этот параметр изменяет размер стрелочки. Не знаю какая единица измерения, но диапазон 0-1.5 юнита.

hide-cursors-checkbox = Отключить курсоры других игроков.
hide-cursors-checkbox-tooltip = Иногда можно перепутать курсоры других игроков со своим. Этой галочкой можно отключить их.
