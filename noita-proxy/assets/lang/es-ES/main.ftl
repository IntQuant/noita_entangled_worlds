connect_steam = Conectarse con steam
connect_steam_create = Crear sala
connect_steam_connect = Conectarse a la sala en el portapapeles
connect_steam_workaround_label = Conectate pegando la id de la sala en este campo: (Por si acaso estas utilizando Wayland y no te puedes conectar de normal)
connect_steam_connect_2 = Conectarse a la sale en el campo de texto
connect_steam_connect_invalid_lobby_id = El portapapeles no contiene un Código de sala

connect_ip = Conectarse por ip

lang_picker = Escoje un lenguaje

button_confirm = Confirmar
button_continue = Continuar
button_retry = Reintentar
button_select_again = Vuelve a seleccionar
button_back = Atrás

button_set_lang = Escoje un lenguaje
button_open_discord = Servidor de Discord

modman = Gestor de Mods
modman_found_automatically = Ruta encontrada automáticamente
modman_use_this = Usar esta ruta
modman_select_manually = Seleccionar manualment
modman_path_to_exe = Selecciona la direccion de noita.exe
modman_invalid_path = Esta ruta es invalida
modman_downloading = Descargando mod...
modman_receiving_rel_info = Recibiendo informacion de lanzamiento...
modman_unpacking = Desempaquetando el mod...
modman_installed = Mod instalado correctamente!
modman_will_install_to = El proxy instalará el mod en:
modman_another_path = Seleccionar ruta alternativa

player_host = Anfitrión
player_me = Yo
player_player = Jugador

version_latest = (Última)
version_check_failed = (Error al buscar actualizaciones)
version_checking = (Buscando actualizaciones)
version_new_available = Actualización para la versión { $new_version } disponible

selfupdate = Actualizar automáticamente
selfupdate_confirm = Confirmar actualización
selfupdate_receiving_rel_info = Receiving información de lanzamiento...
selfupdate_updated = Proxy actualizado! Reinicialo.
selfupdate_unpacking = Desempaquetando...

noita_not_yet = Aún no esta preparado. Porfavor espera antes de lanzar Noita.
noita_can_connect = Esperando conexión con Noita. Hora de iniciar una nueva partida en noita!
noita_connected = Instancia local de Noita conectada.

netman_save_lobby = Copiar id de sala al portapapeles
netman_show_settings = Mostrar menú de opciones
netman_apply_settings = Aplicar configuración a la siguiente partida
apply_default_settings = Restaur configuraciones predeterminadas

ip_note = Nota: la conexión mediante Steam es más fiable. Usala siempre que sea posible.
ip_connect = Connectarse a la IP
ip_host = Crear sala

error_occured = Ha ocurrido un error
error_missing_version_field = Campo de versión en la sala Faltante, La sala ha sido creada con una versión más antigua del Proxy. 
error_version_mismatch = La sala ha sido creada con una version del proxy diferente: { $remote_version }. Tu tienes la { $current_version } instalada.
error_lobby_does_not_exist = Sala inexistente, asegurate de que tus opciones de Red Encima de la seleccion de color de Mina Son iguales a las del Anfitrión.

launcher_already_started = Noita ha sido iniciado.
launcher_no_command = No se pudo iniciar Noita: comando de inicio faltante.
launcher_no_command_2 = Puedes especificar un comando de inicio con el argumento --launch-cmd <command>
launcher_no_command_3 = Puedes poner `noita-proxy --launch-cmd "%command%"` en los argumentos de inicio de Noita en Steam para interceptar  el comando que Steam usa para iniciar el juego.
launcher_start_game = Iniciar Noita
launcher_end_run = Terminar partida
launcher_end_run_confirm = Confirmar
launcher_only_when_awaiting = Solo se puede lanzar el juego en el estado "waiting for noita connection".

connect_settings = Opciones del juego
connect_settings_debug = Opciones de depuración
connect_settings_debug_en = Modo depuración/trampas
connect_settings_debug_fixed_seed = Usar semilla fija
connect_settings_seed = Semilla:
connect_settings_max_players = Máximo de jugadores
connect_settings_wsv = Version de sincronización del mundo a usar:
connect_settings_player_tether = Habilitar correa entre jugadores
connect_settings_player_tether_desc = Correa entre jugadores: Teletransporta a los invitados al Anfitrión si se alejan los suficiente.
connect_settings_player_tether_length = Lonjitud de la correa
connect_settings_item_dedup = Duplicar objetos de la generación del mundo (sincronización).
connect_settings_enemy_hp_scale = Escalado de vida enemiga.
connect_settings_local = Opciones locales
connect_settings_autostart = Iniciar el juego automáticamente

## Game settings

Player-have-same-starting-loadout = Igualar equipamiento inicial
connect_settings_spacewars = Habilitar el uso de Steam aunque no tengas el juego en Steam, por si lo tienes en gog. Todos los jugadores tienen que habilitar esta opción, reinicia el Proxy para que funcione.
Health-per-player = Vida por jugador
Enable-friendly-fire = Habilitar fuego amigo, habilita la elección de equipos en las opciones del juego
Have-perk-pools-be-independent-of-each-other = Hacer que las barajas de atributos sean independientes entre ellas
Amount-of-chunks-host-has-loaded-at-once-synced-enemies-and-physics-objects-need-to-be-loaded-in-by-host-to-be-rendered-by-clients = Cantidad de chunks el anfitrión tiene cargado a la vez, los enemigos sincronizados y los objetos con física tienen que estar cargados por el anfitrión para que los invitados los puedan renderizar
local_health_desc_1 = Cada jugador tiene sus propios Puntos de Vida, la partida se acaba cuando todos los jugadores están muertos.
local_health_desc_2 = Se añade una mecánica de resurrección
Health-percent-lost-on-reviving = Porcentaje de vida perdida al revivir
global_hp_loss = Pérdida de vida global
no_material_damage = Deshabilitar daño por material
perma_death = Muerte permanente
physics_damage = Daño por física
shared_health_desc_1 = Puntos de vida compartidos escalando con cantidad de jugadores.
shared_health_desc_2 = Daño por porcentajes y curas completas ajustados.
shared_health_desc_3 = El modo original.
Local-health = Vida individual
Local-health-alt = Vida individual alternativa
Local-health-perma = Vida individual, muerte permanente
Shared-health = Vida compartida
Game-mode = Modo de juego
world-sync-is-pixel-sync-note = Nota: Sincronización de mundo se refiere a la parte que sincroniza los pixeles del mundo. Los enemigos y otras entidades se veran inafectados.
Higher-values-result-in-less-performance-impact = Valores mas altos pueden resultar en un impacto al rendimiento.
World-will-be-synced-every-this-many-frames = El mundo se sincronizara cada x Frames.

## Savestate

New-game = Nueva partida
Continue = Continuar
savestate_desc = Se ha detectado un estado de guardado de una partida anterior. Desea continuar esa partida o Iniciar una nueva (y sobreescribir el estado de guardado)?
An-in-progress-run-has-been-detected = Se ha detectado una partida en progreso.

## Player appearance

Gem = Gema
Amulet = Amuleto
Crown = Corona
Cape-edge-color = Color borde capa
Cape-color = Color capa
Forearm-color = Color antebrazo
Arm-color = Color brazo
Alt-color = Color secundario
Main-color = Color principal
Reset-colors-to-default = Restaurar colores a predeterminados
Shift-hue = Cambiar tono

## Connected

Show-debug-info = Mostrar información de depuración/conexión
hint_spectate = Usa [',' o d-pad-left] y ['.' o d-pad-right] pra espectar a otros jugadores. '/' para voler a ti
hint_ping = [Botón medio del ratón o R3] crea un ping
Show-debug-plot = Mostrar grafica de depuración
Record-everything-sent-to-noita = Guardar TODO lo enviado a noita.

## IP Connect

ip_could_not_connect = No se pudo conectar
ip_wait_for_connection = Conectandose a la IP...
## Info

info_stress_tests = Estamos haciendo salas publicas (osea pruebas de estrés) cada sábado a las 18:00 UTC. Únete a nuestro discord para más información.
Info = información
## Local settings

connect_settings_random_ports = Usar un puerto no estandar. Puede ser más robusto y permite lanzar varios proxies en un mismo ordenador pero Noita necesitará ser lanzado mediante el proxy.

## UX settings

ping-note = Parámetros del ping
ping-lifetime = Duración del ping en segundos.
ping-scale = Tamaño del ping.
ping-lifetime-tooltip = Este parametro cambia cuantos frames (segundos*60, porque se supone que el juego debe correr a 60fps?) el ping duroa. Rango: 0-60 segundos.
hide-cursors-checkbox = Deshabilitar los cursores de otros jugadores
hide-cursors-checkbox-tooltip = A veces puedes confundir otros cursores con el tuyo, si es tu caso puedes deshabilitarlos por completo con esta opción.
## Steam connect

Switch-mode-and-restart = Cambiar modo y reiniciar
Make-lobby-public = Hacer la sala pública
## Lobby list

Open-lobby-list = Abrir lista de salas
Only-EW-lobbies = Solo salas de EW
Join = Unirse
Not-Entangled-Worlds-lobby = La sala no es de Entangled Worlds
No-public-lobbies-at-the-moment = Actualmente no hay salas públicas :(
Lobby-list-pending = Lista de salas pendiente...
Refresh = Refrescar
Lobby-list = Lista de salas

## Gamemode names

game_mode_Shared = Vida compartida
game_mode_LocalNormal = Vida individual
game_mode_LocalPermadeath = Vida individual (Muerte permanente)
game_mode_LocalAlternate = Vida individual (Alternativo)
game_mode_PvP = PvP
