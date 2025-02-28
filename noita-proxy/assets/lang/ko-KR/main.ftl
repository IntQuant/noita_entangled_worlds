connect_steam = 스팀으로 연결하기
connect_steam_create = 로비 생성하기
connect_steam_connect = 클립보드의 로비에 참가하기
connect_steam_workaround_label = 여기에 로비 ID를 붙여넣어서 참가하기(Wayland를 사용 중이고 일반적인 방식으로 되지 않을 경우)
connect_steam_connect_2 = 텍스트 입력창에 적힌 로비에 참가하기
connect_steam_connect_invalid_lobby_id = 클립보드에 로비 코드가 없음

connect_ip = IP로 참가하기

lang_picker = 언어를 선택하세요

button_confirm = 확인
button_continue = 계속
button_retry = 재시도
button_select_again = 다시 확인
button_back = 뒤로

button_set_lang = 언어 선택
button_open_discord = 디스코드 서버

modman = 모드 관리자
modman_found_automatically = 자동으로 경로를 찾았습니다:
modman_use_this = 이 경로 사용하기
modman_select_manually = 수동으로 선택하기
modman_path_to_exe = noita.exe 경로를 선택하세요
modman_invalid_path = 올바르지 않은 경로입니다
modman_downloading = 모드 다운로드 중...
modman_receiving_rel_info = 릴리즈 정보 가져오는 중...
modman_unpacking = 모드 압축 푸는 중...
modman_installed = 모드가 설치되었습니다!
modman_will_install_to = 프록시가 다음 경로에 모드를 설치합니다:
modman_another_path = 다른 경로 선택하기

player_host = 호스트
player_me = 나
player_player = 플레이어

version_latest = (최신 버전)
version_check_failed = (업데이트 확인 불가)
version_checking = (업데이트 확인 중)
version_new_available = { $new_version }으로 업데이트 가능

selfupdate = 자동 업데이트
selfupdate_confirm = 업데이트 확인
selfupdate_receiving_rel_info = 릴리즈 정보 가져오는 중...
selfupdate_updated = 프록시가 업데이트되었습니다! 지금 재시작 하세요.
selfupdate_unpacking = 압축 푸는 중...

noita_not_yet = 아직 시작할 준비가 되지 않았습니다. 노이타를 시작하기 전에 기다리세요.
noita_can_connect = 노이타 연결을 기다리는 중. 노이타에서 새 게임을 시작할 시간이에요!
noita_connected = 로컬 노이타 인스턴스 연결됨.

netman_save_lobby = 로비 ID 클립보드에 복사하기
netman_show_settings = 설정 화면 보이기
netman_apply_settings = 다음 런에 설정 적용하기
apply_default_settings = 설정은 기본값으로 되돌리기

ip_note = 참고: 스팀 네트워킹이 더 확실하니, 가능하면 사용하시기 바랍니다.
ip_connect = IP에 연결하기
ip_host = 서버 생성하기

error_occured = 다음 오류가 발생했습니다:
error_missing_version_field = 로비에 버전 정보가 없습니다. 이 로비는 오래된 프록시 버전으로 생성되었습니다.
error_version_mismatch = 로비가 다른 버전의 프록시로 생성되었습니다. 로비의 버전은 { $remote_version }이고, 당신의 버전은 { $current_version }입니다.
error_lobby_does_not_exist = 로비가 존재하지 않습니다.

launcher_already_started = 노이타가 이미 실행중입니다.
launcher_no_command = 노이타를 실행할 수 없습니다: 실행 명령어 없음.
launcher_no_command_2 = 실행 명령어는 --launch-cmd <명령어> 로 지정할 수 있습니다.
launcher_no_command_3 = 'noita-proxy --launch-cmd "%명령어%"'를 스팀 실행 옵션에 넣어 스팀의 실행 명령어를 중간에 덮어씌울 수 있습니다.
launcher_start_game = 노이타 실행하기
launcher_end_run = 런 종료
launcher_end_run_confirm = 확인
launcher_only_when_awaiting = "노이타 연결을 기다리는 중" 상태에서만 노이타를 실행할 수 있습니다.

connect_settings = 게임 설정
connect_settings_debug = 디버그 설정
connect_settings_debug_en = 디버그/치트 모드
connect_settings_debug_fixed_seed = 고정 시드 사용
connect_settings_seed = 시드:
connect_settings_max_players = 최대 플레이어
connect_settings_wsv = 사용할 월드 싱크 버전:
connect_settings_player_tether = 플레이어 연결줄 활성화됨
connect_settings_player_tether_desc = 플레이어 연결줄: 플레이어가 호스트에서 너무 멀어질 경우 호스트로 순간이동합니다.
connect_settings_player_tether_length = 연결줄 길이
connect_settings_item_dedup = 월드 생성으로 생겨난 아이템들을 복제되지 않게(동기화) 합니다.
connect_settings_enemy_hp_scale = 적 체력 비율.
connect_settings_local = 로컬 설정
connect_settings_autostart = 노이타를 자동으로 시작

## Game settings

Player-have-same-starting-loadout = Player have same starting loadout
connect_settings_spacewars = 노이타를 스팀에서 소유하지 않아도 스팀 네트워킹을 사용하게 합니다.GOG 버전이라면 사용하세요. 모든 플레이어가 켜놓아야 합니다. 프록시를 재시작해서 적용하세요.
Health-per-player = 플레이어당 체력
Enable-friendly-fire = 아군 오사를 활성화하고, 설정에서 팀을 고르게 합니다
Have-perk-pools-be-independent-of-each-other = 퍽(능력)풀이 플레이어마다 다릅니다
Amount-of-chunks-host-has-loaded-at-once-synced-enemies-and-physics-objects-need-to-be-loaded-in-by-host-to-be-rendered-by-clients = 호스트가 한번에 불러와두는 청크의 양. 동기화된 적과 물리적 물체들은 호스트가 불러와야 플라이언트가 볼 수 있습니다
local_health_desc_1 = 모든 플레이어가 각자 체력을 가지고, 모든 플레이어가 사망하면 런이 종료됩니다.
local_health_desc_2 = 리스폰을 하는 방법이 있습니다.
Health-percent-lost-on-reviving = 다시 살아났을 때 잃는 최대 체력 퍼센트
global_hp_loss = 체력 손실 공유
no_material_damage = 물질로 인한 피해 없음
perma_death = 퍼마데스
physics_damage = 물리 피해
shared_health_desc_1 = 체력을 공유하지만, 플레이어의 수에 따라갑니다.
shared_health_desc_2 = 체력 비례 피해와 완전 치유의 효과가 조정됩니다.
shared_health_desc_3 = 원본 모드.
Local-health = 개별 체력
Local-health-alt = 개별 체력 대체 옵션
Local-health-perma = 개별 체력 퍼마데스
Shared-health = 공유된 체력
Game-mode = 게임모드
world-sync-is-pixel-sync-note = 참고: 월드 동기화는 월드의 픽셀(물질)을 동기화하는것을 말합니다. 적과 다른 엔티티는 영향을 받지 않습니다.
Higher-values-result-in-less-performance-impact = 값이 높아질수록 성능에 영향을 줍니다.
World-will-be-synced-every-this-many-frames = 월드가 매 이 프레임마다 동기화됩니다.

## Savestate

New-game = 새 게임
Continue = 계속하기
savestate_desc = 진행중이던 런의 저장 파일을 감지했습니다. 그 런을 계속하시겠습니까, 아니면 새 게임을 시작해 저장 파일을 초기화하시겠습니까?
An-in-progress-run-has-been-detected = 진행중인 런이 감지되었습니다.

## Player appearance

Gem = 보석
Amulet = 애뮬릿
Crown = 왕관
Cape-edge-color = 망토 가장자리 색
Cape-color = 망토 색
Forearm-color = 팔뚝 색
Arm-color = 팔 색
Alt-color = 보조 색
Main-color = 주 색
Reset-colors-to-default = 기본 색으로 초기화
Shift-hue = 색조 변경

## Connected

Show-debug-info = 디버그/연결 정보 표시
hint_spectate = [','나 십자키 왼쪽] 혹은 ['.'나 십자키 오른쪽]을 눌러 다른 플레이어를 관전하거나 '/'를 눌러 자신을 관전하세요
hint_ping = [마우스 휠이나 오른쪽 스틱]을 눌러 핑 찍기
Show-debug-plot = 디버그 플롯 표시
Record-everything-sent-to-noita = 노이타로 보낸 것을 전부 기록합니다.

## IP Connect

ip_could_not_connect = 연결할 수 없습니다
ip_wait_for_connection = IP로 연결 중...
## Info

info_stress_tests = 매주 토요일 UTC 18:00에 공개 로비로 스트레스 테스트를 진행합니다. 디스코드에 들어와 더 많은 정보를 확인하세요.
Info = 정보
## Local settings

connect_settings_random_ports = 미리 정해둔 포트를 사용하지 않습니다. 신뢰성이 좋아지고 한 컴퓨터에서 여러개의 프록시를 실행할 수 있게 되지만, 노이타를 프록시를 통해서 실행해야만 합니다.

## UX settings

ping-note = 핑 화살표 설정
ping-lifetime = 핑 화살표의 지속시간.(초)
ping-scale = 핑 화살표 크기.
ping-lifetime-tooltip = 이 값은 화살표가 얼마나 오래 지속되는가를 결정합니다. 0-60초 사이.
ping-scale-tooltip = 이 값은 화살표의 크기를 결정합니다. 단위는 무엇인지 모르겠지만, 0-1.5 사이입니다.

hide-cursors-checkbox = 다른 플레이어의 커서 비활성화
hide-cursors-checkbox-tooltip = 가끔씩 다른 플레이어의 커서를 자신의 것과 헷갈릴 수 있기에, 이 설정을 활성화 해 전부 비활성화할 수 있습니다.

## Steam connect

Make-lobby-public = Make lobby public
## Lobby list

Open-lobby-list = Open lobby list
Only-EW-lobbies = Only EW lobbies
Join = Join
Not-Entangled-Worlds-lobby = Not Entangled Worlds lobby
No-public-lobbies-at-the-moment = No public lobbies at the moment :(
Lobby-list-pending = Lobby list pending...
Refresh = Refresh
Lobby-list = Lobby list