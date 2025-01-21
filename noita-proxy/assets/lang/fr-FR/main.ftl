connect_steam = Connexion via Steam
connect_steam_create = Créer un lobby
connect_steam_connect = Se connecter au lobby dans le presse-papiers
connect_steam_workaround_label = Connectez-vous en collant l'ID du lobby dans ce champ: (Au cas où vous utiliseriez Wayland et la méthode normal ne fonctionne pas)
connect_steam_connect_2 = Connectez-vous au lobby dans le champ de texte
connect_steam_connect_invalid_lobby_id = Presse papier ne contient pas de code de lobby

connect_ip = Connexion via IP

lang_picker = Choisissez une langue

button_confirm = Confirmer
button_continue = Continuer
button_retry = Réessayer
button_select_again = Sélectionner à nouveau
button_back = Retour

button_set_lang = Sélectionner une langue
button_open_discord = Serveur Discord

modman = Mod Manager
modman_found_automatically = Chemin trouvé automatiquement:
modman_use_this = Utiliser celui-ci
modman_select_manually = Sélectionner manuellement
modman_path_to_exe = Sélectionner le chemin vers noita.exe
modman_invalid_path = Ce Chemin n'est pas valide
modman_downloading = Téléchargement du mod...
modman_receiving_rel_info = Réception des information de release...
modman_unpacking = Décompression du mod...
modman_installed = Le Mod a été installé!
modman_will_install_to = Proxy va installer le mod dans:
modman_another_path = Sélectionner un chemin différent

player_host = Hôte
player_me = Moi
player_player = Joueur

version_latest = (dernière)
version_check_failed = (vérification de mise à jour échoué)
version_checking = (vérification de mise à jour)
version_new_available = Mise à jour disponible à { $new_version }

selfupdate = Mise à jour automatique
selfupdate_confirm = Confirmer la mise à jour
selfupdate_receiving_rel_info = Réception des information de release...
selfupdate_updated = Proxy mis à jour! Redémarrage imminent.
selfupdate_unpacking = Décompression...

noita_not_yet = Toujours pas prêt, veuillez attendre avant de lancer Noita.
noita_can_connect = En attente d'une connexion à Noita. Il est temps de commencer de nouvelles parties dans Noita!
noita_connected = Connecté à l'instance de Noita locale.

netman_save_lobby = Sauvegarder l'ID du lobby dans le presse-papiers
netman_show_settings = Afficher les options
netman_apply_settings = Appliquer les paramètres pour être utilisé dans la prochaine partie
apply_default_settings = Réinitialiser les paramètres

ip_note = Note: Steam Networking est plus fiable. Utilisez-le si possible.
ip_connect = Connexion à l'IP
ip_host = Créer un serveur

error_occured = Une Erreur est survenue:
error_missing_version_field = Ce lobby n'a pas de champ de version. Il a été créé par une version antérieure du Proxy.
error_version_mismatch = Ce lobby a été créé par un Proxy de version différente: { $remote_version }. Vous avez la version { $current_version } d'installée.
error_lobby_does_not_exist = Le lobby n'existe pas. Assurez-vous que vos paramètres de Steam Network au-dessus de la sélection de couleur soient identiques à celle de l'hôte.

launcher_already_started = Noita est déjà lancé.
launcher_no_command = Impossible de lancer Noita: aucune commande de lancement.
launcher_no_command_2 = La commande de lancement peut être spécifiée avec l'option: --launch-cmd 
launcher_no_command_3 = Vous pouvez insérer `noita-proxy --launch-cmd "%command%"` dans les options de lancement Steam pour intercepter la commande qu'utilise Steam pour lancer le jeu.
launcher_start_game = Lancer Noita
launcher_end_run = Terminer la partie
launcher_end_run_confirm = Confirmer
launcher_only_when_awaiting = Peut seulement lancer le jeu dans l'état "en attente de connexion à Noita".

connect_settings = Options du Jeu
connect_settings_debug = Options de Débug
connect_settings_debug_en = Mode de Triche/Débug
connect_settings_debug_fixed_seed = Utiliser une seed fixé
connect_settings_seed = Seed:
connect_settings_max_players = Joueurs maximum
connect_settings_wsv = Version de synchronisation du monde à utiliser:
connect_settings_player_tether = Attache des joueurs activée
connect_settings_player_tether_desc = Attache des joueurs: Téléporte les clients à l'hôte s'ils sont trop loin.
connect_settings_player_tether_length = Taille de l'attache
connect_settings_item_dedup = Dé-duplique (synchronisation) les objets spawnés par la génération du monde.
connect_settings_enemy_hp_scale = Mise à l'échelle des Points de Vie ennemies.
connect_settings_local = Options Locales
connect_settings_autostart = Lancer le jeu automatiquement

## Game settings

Player-have-same-starting-loadout = Player have same starting loadout
connect_settings_spacewars = Autorise à utiliser le Steam network même s'ils n'ont pas le jeu sur Steam, dans le cas où vous avez la version GOG du jeu. Tous les joueurs ont besoin d'avoir cette option d'activer pour qu'elle marche, relancer votre Proxy pour qu'elle prenne effet.
Health-per-player = Points de Vie par joueur
Enable-friendly-fire = Activer le tir allié, permet la création d'équipe dans les paramètres du jeu
Have-perk-pools-be-independent-of-each-other = Séparer la sélection des avantages de chaque joueur
Amount-of-chunks-host-has-loaded-at-once-synced-enemies-and-physics-objects-need-to-be-loaded-in-by-host-to-be-rendered-by-clients = La quantité de chunks qui ont été chargé par l'hôte d'un coup, synchronisé ses ennemies et objets physiques qui doivent être chargé par l'hôte pour être affiché aux clients
local_health_desc_1 = Tous les joueurs ont leur propre PV, la partie se termine lorsqu'ils sont tous morts.
local_health_desc_2 = Il y a un système de respawn.
Health-percent-lost-on-reviving = Pourcentage de vie max perdu lors d'un respawn
global_hp_loss = Perte de PV partagé
no_material_damage = Pas de dégâts matériels
perma_death = Mort permanente
physics_damage = Dégâts physique
shared_health_desc_1 = Les PV sont partagés, mais augmente en fonction du nombre de joueurs.
shared_health_desc_2 = Dégâts basé sur un pourcentage et les soins complet sont ajustés.
shared_health_desc_3 = Mode d'origine.
Local-health = Points de Vie Local
Local-health-alt = Point de Vie Local alterne
Local-health-perma = Point de Vie Local mort permanente
Shared-health = Point de Vie partagé
Game-mode = Mode de jeu
world-sync-is-pixel-sync-note = Note: Synchronisation du monde signifie la synchronisation des pixels(matériaux) du monde. Les ennemies et autres entités ne sont pas affectées par ça.
Higher-values-result-in-less-performance-impact = Des valeurs plus élevées engendreront une baisse de performance.
World-will-be-synced-every-this-many-frames = Le monde sera synchronisé toutes les tant de frames.

## Savestate

New-game = Nouvelle partie
Continue = Continuer
savestate_desc = Sauvegarde d'une partie précédente a été trouvé. Voulez-vous continuer cette partie, ou commencer une nouvelle partie (et réinitialiser la sauvegarde)?
An-in-progress-run-has-been-detected = Une partie en cours a été trouvé.

## Player appearance

Gem = Gemme
Amulet = Amulette
Crown = Couronne
Cape-edge-color = Couleur du bord de la cape
Cape-color = Couleur de la cape
Forearm-color = Couleur de l'avant-bras
Arm-color = Couleur du bras
Alt-color = Couleur alternative
Main-color = Couleur principale
Reset-colors-to-default = Réinitialiser les couleurs
Shift-hue = Changer de teinte

## Connected

Show-debug-info = Afficher les informations de Débug/Connection
hint_spectate = Utiliser les touches [',' ou d-pad-gauche] et ['.' ou d-pad-droite] pour observer les autres joueurs. '/' pour soi-même
hint_ping = [Clique de la molette ou joystick droit] spawn un ping
Show-debug-plot = Afficher graphe de Débug
Record-everything-sent-to-noita = Enregistrer TOUT envoyé à Noita.

## IP Connect

ip_could_not_connect = Connexion échoué
ip_wait_for_connection = Connexion à l'IP...

## Info

info_stress_tests = Nous organisons des lobbies publique (ie. des tests de stress) tous les samedis à 18:00 UTC. Rejoignez notre Discord pour plus d'information.
Info = Info

## Local settings

connect_settings_random_ports = N'utilisez pas de port prédéterminé. Ça rend les choses un peu plus robustes et permet de lancer plusieurs Proxies sur le même ordinateur, mais Noita devra être lancé par le Proxy.

## UX settings

ping-note = Options de la flèche de ping
ping-lifetime = Durée de vie de la flèche de ping en seconde.
ping-scale = Taille de la flèche de ping.
ping-lifetime-tooltip = Cette option change combien de frames (seconde*60, puisque le jeu est supposé tourner à 60 fps?) la flèche de ping vie. De 0 à 60 secondes.
ping-scale-tooltip = Cette option change la taille de la flèche de ping. Je ne sais pas quelle mesure c'est, mais ça va de 0 à 1,5 unités.

hide-cursors-checkbox = Désactive le curseur des autres
hide-cursors-checkbox-tooltip = Il arrive de confondre le curseur de tes amis avec le vôtre. Dans ce cas, vous pouvez les désactiver avec cette option.
