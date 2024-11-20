connect_steam = Conectar usando steam
connect_steam_create = Criar lobby
connect_steam_connect = Conectar ao lobby na área de transfêrencia
connect_steam_workaround_label = Conecte-se por colar o lobby id nesse campo: (Caso você está usando Wayland e o jeito normal não funciona)
connect_steam_connect_2 = Conecte-se ao lobby do campo de texto
connect_steam_connect_invalid_lobby_id = Área de transferência não contém um código de lobby

connect_ip = Conectar usando ip

lang_picker = Escolha uma linguagem

button_confirm = Confirmar
button_continue = Continuar
button_retry = Tentar novamente
button_select_again = Selecione novamente
button_back = Voltar

button_set_lang = Selecionar linguagem
button_open_discord = Servidor discord

modman = Gerenciador de mods
modman_found_automatically = Caminho encontrado automaticamente:
modman_use_this = Usar esse
modman_select_manually = Selecionar manualmente
modman_path_to_exe = Selecionar caminho para noita.exe
modman_invalid_path = Esse caminho não é valido
modman_downloading = Baixando mod...
modman_receiving_rel_info = Recebendo informações de lançamento...
modman_unpacking = Descompactando mod...
modman_installed = Mod foi instalado!
modman_will_install_to = Proxy vai instalar o mod em:
modman_another_path = Selecione um caminho diferente

player_host = Host
player_me = Eu
player_player = Jogador

version_latest = (ultima)
version_check_failed = (não foi possível verificar por atualizações)
version_checking = (verificando por autualizações)
version_new_available = Atualização disponível para { $new_version }

selfupdate = Autoatualização
selfupdate_confirm = Confirmar atualização
selfupdate_receiving_rel_info = Recebendo informações de lançamento...
selfupdate_updated = Proxy atualizado! Reinicie-o agora.
selfupdate_unpacking = Descompactando...

noita_not_yet = Ainda não está pronto. Por favor aguarde antes de iniciar Noita.
noita_can_connect = Esperando conexão Noita. Agora é hora de iniciar um novo jogo Noita!
noita_connected = Instância local Noita conectada.

netman_save_lobby = Salvar id do lobby para área de transferência
netman_show_settings = Mostrar tela de configurações
netman_apply_settings = Aplicar configurações para serem usadas na próxima partid
apply_default_settings = Resetar configurações para o padrão

ip_note = Nota: rede steam é mais confiável. Use ela, se possivel.
ip_connect = Conectar ao IP
ip_host = Criar um servidor

error_occured = Um erro ocorreu:
error_missing_version_field = Lobby não tem um campo de versão. O lobby foi criado por uma versão antiga do proxy.
error_version_mismatch = Lobby foi criado por um proxy com uma versão diferente: { $remote_version }. Você tem { $current_version } atualmente instalada.
error_lobby_does_not_exist = Lobby não existe.

launcher_already_started = Noita já está aberto.
launcher_no_command = Não pode iniciar noita: sem comando de inicialização
launcher_no_command_2 = Comando de inicialização pode ser especificado com a opção --launch-cmd <comando>.
launcher_no_command_3 = Você pode colocar `noita-proxy --launch-cmd "%command%"` nas opções de inicialização da steam para intereceptar qualquer comando que steam use para iniciar o jogo.
launcher_start_game = Iniciar noita
launcher_end_run = Finalizar partida
launcher_end_run_confirm = Confirmar
launcher_only_when_awaiting = Pode iniciar o jogo apenas no estado "esperando por conexão noita".

connect_settings = Configurações de jogo
connect_settings_debug = Configurações de depuração
connect_settings_debug_en = Modo Depuração/Trapaças
connect_settings_debug_fixed_seed = Usar semente fixa
connect_settings_seed = semente:
connect_settings_max_players = Máximo de jogadores
connect_settings_wsv = Versão de sincronização de mundo para usar:
connect_settings_player_tether = Tether ativada
connect_settings_player_tether_desc = Tether de jogador: Teleporta clientes para o host se ficarem longe o suficiente.
connect_settings_player_tether_length = Comprimento de tether
connect_settings_item_dedup = Deduplicar (sincronizar) itens spawnado pela geração de mundo.
connect_settings_enemy_hp_scale = Escala de vida de inimigos.
connect_settings_local = Configurações locais
connect_settings_autostart = Iniciar o jogo automaticamente

## Game settings

connect_settings_spacewars = Permitir usar rede steam mesmo se você não tem o jogo na steam, caso você tenha a versão gog do jogo. Todos jogadores precisam disso ativado para funcionar, reinicie o proxy pra fazer efeito
Health-per-player = Vida por jogador
Enable-friendly-fire = Ativar fogo amigo, permite escolher times no lobby
Have-perk-pools-be-independent-of-each-other = Fazer escolha de perks ser independente de uma a outra
Amount-of-chunks-host-has-loaded-at-once-synced-enemies-and-physics-objects-need-to-be-loaded-in-by-host-to-be-rendered-by-clients = Quantidade de chunks que host tem carregado ao mesmo tempo, inimigos sincronizados e objetos físicos precisam ser carregados pelo host para ser renderizado pelos clientes
local_health_desc_1 = Todo jogador tem a própria vida, partida termina quando todos jogadores estão mortos.
local_health_desc_2 = Tem uma mecânica de reviver.
Health-percent-lost-on-reviving = Porcentagem de vida máxima perdida ao reviver
global_hp_loss = Perder vida globalmente
no_material_damage = Sem dano material
perma_death = Morte permanente
physics_damage = Dano físico
shared_health_desc_1 = Vida é compartilhada, mas escala com quantidade de jogadores.
shared_health_desc_2 = Dano baseado em porcentagem e curas completas são ajustadas.
shared_health_desc_3 = O modo original.
Local-health = Vida local
Shared-health = Vida compartilhada
Game-mode = Modo de jogo
world-sync-is-pixel-sync-note = Nota: Sincronização de mundo refere a parte que sincroniza os pixels(materiais) do mundo. Inimigos e outras entidades não são afetados por isso.
Higher-values-result-in-less-performance-impact = Valores maiores resultam em um impacto de desempenho menor.
World-will-be-synced-every-this-many-frames = Mundo será sincronizado a cada esse tanto de frames.

## Savestate

New-game = Novo jogo
Continue = Continuar
savestate_desc = Estado de salvamento de uma partida anterior detectado. Você deseja continuar essa partida, ou iniciar um novo jogo (e resetar o progresso)?
An-in-progress-run-has-been-detected = Uma partida em progresso foi detectada.

## Player appearance

Gem = Gema
Amulet = Amuleto
Crown = Coroa
Cape-edge-color = Cor da borda da capa
Cape-color = Cor da capa
Forearm-color = Cor do antebraço
Arm-color = Cor do braço
Alt-color = Cor alternativa
Main-color = Cor principal
Reset-colors-to-default = Resetar cores para o padrão
Shift-hue = Mudar matiz

## Connected

Show-debug-info = Mostrar informações de Depuração/conexão
hint_spectate = Use [',' ou botão direcional esquerda] e ['.' ou botão direcional direita] para assitir outros jogadores. '/' para si mesmo
hint_ping = [Botão do meio do mouse ou analógico direito] criar sinalização
Show-debug-plot = Mostrar plot de depuração
Record-everything-sent-to-noita = Gravar TUDO enviado pro noita.

## IP Connect

ip_could_not_connect = Não foi possível conectar.
ip_wait_for_connection = Conectando ao ip...
## Info

info_stress_tests = Estamos fazendo lobbies públicos (vulgo testes de estresse) todo sábado, 18:00 UTC. Entre em nosso discord para mais informação.
Info = Informação
## Local settings

connect_settings_random_ports = Não use uma porta predeterminada. Faz coisas um pouco mais robustas e permite multiplos proxies para serem iniciados no mesmo computador, mas Noita terá que ser iniciado via proxy.