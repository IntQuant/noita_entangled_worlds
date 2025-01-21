connect_steam = Verbindung per Steam herstellen
connect_steam_create = Erstelle eine Lobby
connect_steam_connect = Mit Lobby aus der Zwischenablage verbinden
connect_steam_workaround_label = Verbinden, indem Sie die Lobby-ID in dieses Feld einfügen: (Falls Sie Wayland verwenden und der normale Weg nicht funktioniert)
connect_steam_connect_2 = Mit Lobby im Textfeld verbinden
connect_steam_connect_invalid_lobby_id = Die Zwischenablage enthält keinen gültigen Lobby-Code

connect_ip = Per IP verbinden

lang_picker = Sprache wählen

button_confirm = Bestätigen
button_continue = Fortfahren
button_retry = Erneut versuchen
button_select_again = Erneut auswählen
button_back = Zurück

button_set_lang = Sprache auswählen
button_open_discord = Discord Server

modman = Mod-Manager
modman_found_automatically = Ein Spiel-Pfad wurde automatisch gefunden:
modman_use_this = Diesen Pfad verwenden
modman_select_manually = Pfad Manuell auswählen
modman_path_to_exe = Pfad zu noita.exe auswählen
modman_invalid_path = Dieser Pfad ist ungültig
modman_downloading = Mod wird heruntergeladen...
modman_receiving_rel_info = Release-Informationen werden empfangen...
modman_unpacking = Mod wird entpackt...
modman_installed = Mod wurde installiert!
modman_will_install_to = Proxy installiert die Mod in:
modman_another_path = Einen anderen Pfad auswählen

player_host = Host
player_me = Ich
player_player = Spieler

version_latest = (neuste)
version_check_failed = (Updates konnten nicht überprüft werden)
version_checking = (Updates werden überprüft)
version_new_available = Update verfügbar auf { $new_version }

selfupdate = Selbst-Update
selfupdate_confirm = Update bestätigen
selfupdate_receiving_rel_info = Release-Informationen werden empfangen...
selfupdate_updated = Proxy aktualisiert! Bitte starten Sie jetzt neu.
selfupdate_unpacking = Entpacken...

noita_not_yet = Noch nicht bereit. Bitte warten Sie, bevor Sie Noita starten.
noita_can_connect = Auf Noita-Verbindung warten. Sie können jetzt ein neues Spiel in Noita starten!
noita_connected = Lokale Noita-Instanz verbunden.

netman_save_lobby = Lobby-ID in die Zwischenablage speichern
netman_show_settings = Einstellungen anzeigen
netman_apply_settings = Einstellungen für den nächsten Start übernehmen
apply_default_settings = Setzt die Einstellungen auf Standard zurück

ip_note = Hinweis: Verbindung per Steam ist zuverlässiger.
ip_connect = Per IP verbinden
ip_host = Einen Server erstellen

error_occured = Ein Fehler ist aufgetreten:
error_missing_version_field = Die Lobby hat kein Versionsfeld. Die Lobby wurde von einer älteren Proxy-Version erstellt.
error_version_mismatch = Die Lobby wurde mit einer anderen Proxy-Version erstellt: { $remote_version }. Sie haben derzeit { $current_version } installiert.
error_lobby_does_not_exist = Lobby existiert nicht.

launcher_already_started = Noita ist bereits gestartet.
launcher_no_command = Noita kann nicht gestartet werden: Kein Startbefehl vorhanden.
launcher_no_command_2 = Ein Startbefehl kann mit der Option --launch-cmd <Befehl> angegeben werden.
launcher_no_command_3 = Sie können `noita-proxy --launch-cmd "%command%"` in die Startoptionen von Steam einfügen, um den von Steam verwendeten Startbefehl abzufangen.
launcher_start_game = Noita starten
launcher_end_run = Spiel beenden
launcher_end_run_confirm = Bestätigen
launcher_only_when_awaiting = Das Spiel kann nur im Zustand "Auf Noita-Verbindung warten" gestartet werden.

connect_settings = Spieleinstellungen
connect_settings_debug = Debug-Einstellungen
connect_settings_debug_en = Debug-/Cheat-Modus
connect_settings_debug_fixed_seed = Feste Seed verwenden
connect_settings_seed = Seed:
connect_settings_max_players = Maximale Spieleranzahl
connect_settings_wsv = Zu verwendende World-Sync-Version:
connect_settings_player_tether = Spielerbindung aktiviert
connect_settings_player_tether_desc = Spielerbindung: Teleportiert Clients zum Host, wenn sie sich zu weit entfernen.
connect_settings_player_tether_length = Bindungsreichweite
connect_settings_item_dedup = Duplizierte (generierte) Gegenstände synchronisieren
connect_settings_enemy_hp_scale = Lebenspunkte der Gegner skalieren.
connect_settings_local = Lokale Einstellungen
connect_settings_autostart = Spiel automatisch starten

## Game settings

Player-have-same-starting-loadout = Player have same starting loadout
connect_settings_spacewars = Ermöglicht die Verwendung von Steam-Netzwerken, auch wenn Sie das Spiel nicht auf Steam besitzen (z. B. GOG-Version). Alle Spieler müssen dies aktivieren, damit es funktioniert. Proxy muss danach neu gestartet werden.
Health-per-player = Gesundheit pro Spieler
Enable-friendly-fire = Freundliches Feuer aktivieren, erlaubt die Teamauswahl in den Spieleinstellungen
Have-perk-pools-be-independent-of-each-other = Perk-Pools unabhängig voneinander halten
Amount-of-chunks-host-has-loaded-at-once-synced-enemies-and-physics-objects-need-to-be-loaded-in-by-host-to-be-rendered-by-clients = Anzahl der geladenen Chunks des Hosts, synchronisierte Gegner und Physikobjekte müssen vom Host geladen sein, um von Clients gerendert zu werden.
local_health_desc_1 = Jeder Spieler hat seine eigene Gesundheit, das Spiel endet, wenn alle Spieler tot sind.
local_health_desc_2 = Es gibt einen Wiederbelebungsmechanismus.
Health-percent-lost-on-reviving = Prozentualer Maximal-HP-Verlust bei Wiederbelebung
global_hp_loss = HP global verlieren
no_material_damage = Kein Materialschaden
perma_death = Permanente Tode
physics_damage = Physischer Schaden
shared_health_desc_1 = Gesundheit wird geteilt, skaliert aber mit der Spieleranzahl.
shared_health_desc_2 = Prozentbasierter Schaden und vollständige Heilungen werden angepasst.
shared_health_desc_3 = Der ursprüngliche Modus.
Local-health = Lokale Gesundheit (Boss Wiederbelebung)
Local-health-alt = Alternative lokale Gesundheit (Mitnahme Wiederbelebung)
Local-health-perma = Permanente lokale Gesundheit 
Shared-health = Geteilte Gesundheit
Game-mode = Spielmodus
world-sync-is-pixel-sync-note = Hinweis: World-Sync synchronisiert die Pixel (Materialien) der Welt. Gegner und andere Entitäten sind davon nicht betroffen.
Higher-values-result-in-less-performance-impact = Höhere Werte verringern die Leistungsbelastung.
World-will-be-synced-every-this-many-frames = Welt wird alle x Frames synchronisiert.

## Savestate

New-game = Neues Spiel
Continue = Fortfahren
savestate_desc = Ein Speicherstand von einem vorherigen Durchlauf wurde erkannt. Möchten Sie diesen fortsetzen oder ein neues Spiel starten (und den Speicherstand zurücksetzen)?
An-in-progress-run-has-been-detected = Ein laufendes Spiel wurde erkannt.

## Player appearance

Gem = Edelstein
Amulet = Amulett
Crown = Krone
Cape-edge-color = Farbe des Umhangrandes
Cape-color = Umhangfarbe
Forearm-color = Unterarmfarbe
Arm-color = Armfarbe
Alt-color = Alternativfarbe
Main-color = Hauptfarbe
Reset-colors-to-default = Farben auf Standard zurücksetzen
Shift-hue = Färbung verschieben

## Connected

Show-debug-info = Debug-/Verbindungsinfo anzeigen
hint_spectate = Verwenden Sie [',' oder Steuerkreuz links] und ['.' oder Steuerkreuz rechts], um andere Spieler zu beobachten. '/' für sich selbst
hint_ping = [Mittlere Maustaste oder rechter Joystick] erzeugt einen Ping
Show-debug-plot = Debug-Diagramm anzeigen
Record-everything-sent-to-noita = ALLES aufzeichnen, das an Noita gesendet wird.

## IP Connect

ip_could_not_connect = Verbindung fehlgeschlagen
ip_wait_for_connection = Verbindung zu IP wird hergestellt...

## Info

info_stress_tests = Öffentliche Lobbys (Stresstests) finden jeden Samstag um 18:00 UTC statt. Treten Sie unserem Discord bei, um mehr Informationen zu erhalten.
Info = Info

## Local settings

connect_settings_random_ports = Einen nicht vordefinierten Port verwenden. Erhöht die Robustheit und ermöglicht mehrere Proxys auf demselben Computer, erfordert jedoch, dass Noita über den Proxy gestartet wird.

## UX settings

ping-note = Ping Pfeil Optionen
ping-lifetime = Ping Pfeil Dauer.
ping-scale = Ping-Pfeil Größe.
ping-lifetime-tooltip = Diese Option ändert, wie viele Frames (Sekunden*60, da das Spiel mit 60 FPS laufen soll?) der Ping-Pfeil aktualisiert. Bereich: 0-60.
ping-scale-tooltip = Diese Option ändert die Größe des Ping-Pfeils. Der Bereich liegt bei 0-1,5 Einheiten.

hide-cursors-checkbox = Andere Cursor ausblenden
hide-cursors-checkbox-tooltip = Manchmal kann man die Cursor anderer Spieler mit dem eigenen verwechseln. In diesem Fall können Sie diese deaktivieren.
