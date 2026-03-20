use std::{net::SocketAddr, process::exit, thread::sleep, time::Duration};

use crate::player_cosmetics::player_path;

use tangled::Peer;

use crate::{
    AppSavedState, AudioSettings, PlayerAppearance,
    bookkeeping::{save_paths::SavePaths, save_state::SaveState},
    lobby_code::{LobbyCode, LobbyKind},
    mod_manager::ModmanagerSettings,
    net::{NetManager, NetManagerInit, omni::PeerVariant, steam_networking},
    player_cosmetics::PlayerPngDesc,
    steam_helper,
    util::{args::Args, steam_helper::LobbyExtraData},
};

fn cli_setup(
    args: Args,
) -> (
    Option<steam_helper::SteamState>,
    NetManagerInit,
    LobbyKind,
    AudioSettings,
    steamworks::LobbyType,
) {
    let settings = SavePaths::new().load_settings();
    let saved_state: AppSavedState = settings.app;
    let mut mod_manager: ModmanagerSettings = settings.modmanager;
    let appearance: PlayerAppearance = settings.color;
    let audio: AudioSettings = settings.audio;
    let mut state = steam_helper::SteamState::new(saved_state.spacewars).ok();
    let my_nickname = saved_state
        .nickname
        .unwrap_or("no nickname found".to_string());

    if let Some(state) = &mut state {
        mod_manager.try_find_game_path(Some(state));
    } else if let Some(p) = args.exe_path {
        mod_manager.game_exe_path = p
    } else {
        println!("needs game exe path if you want to join as host")
    }
    mod_manager.try_find_save_path();
    let run_save_state = if let Ok(path) = std::env::current_exe() {
        SaveState::new(path.parent().unwrap().join("save_state"))
    } else {
        SaveState::new("./save_state/")
    };
    let player_path = player_path(mod_manager.mod_path());
    let mut cosmetics = (false, false, false);
    if let Some(path) = &mod_manager.game_save_path {
        let flags = path.join("save00/persistent/flags");
        let hat = flags.join("secret_hat").exists();
        let amulet = flags.join("secret_amulet").exists();
        let gem = flags.join("secret_amulet_gem").exists();
        if !hat {
            cosmetics.0 = false
        }
        if !amulet {
            cosmetics.1 = false
        }
        if !gem {
            cosmetics.2 = false
        }
    }
    let netmaninit = NetManagerInit {
        my_nickname,
        save_state: run_save_state,
        cosmetics,
        mod_path: mod_manager.mod_path(),
        player_path,
        modmanager_settings: mod_manager,
        player_png_desc: PlayerPngDesc {
            cosmetics: cosmetics.into(),
            colors: appearance.player_color,
            invert_border: appearance.invert_border,
        },
        noita_port: 21251,
    };
    (
        state,
        netmaninit,
        if saved_state.spacewars {
            LobbyKind::Gog
        } else {
            LobbyKind::Steam
        },
        audio,
        if saved_state.public_lobby {
            steamworks::LobbyType::Public
        } else if saved_state.allow_friends {
            steamworks::LobbyType::FriendsOnly
        } else {
            steamworks::LobbyType::Private
        },
    )
}

pub fn connect_cli(lobby: String, args: Args) {
    let (state, netmaninit, kind, audio, _) = cli_setup(args);
    let variant = if lobby.contains(':') {
        let p = Peer::connect(lobby.parse().unwrap(), None).unwrap();
        while p.my_id().is_none() {
            sleep(Duration::from_millis(100))
        }
        PeerVariant::Tangled(p)
    } else if let Some(state) = state {
        let peer = steam_networking::SteamPeer::new_connect(
            LobbyCode::parse(lobby.trim()).unwrap().code,
            state.client,
        );
        PeerVariant::Steam(peer)
    } else {
        println!("no steam");
        exit(1)
    };
    let player_path = netmaninit.player_path.clone();
    let netman = NetManager::new(variant, netmaninit, audio);
    netman.start_inner(player_path, Some(kind)).unwrap();
}

/// Bind to the provided `bind_addr` with `args` with CLI output only.
///
/// The `bind_addr` is either `Some` address/port pair to bind to, or `None` to use Steam networking.
pub fn host_cli(bind_addr: Option<SocketAddr>, args: Args) {
    let (state, netmaninit, kind, audio, lobbytype) = cli_setup(args);
    let variant = if let Some(bind_addr) = bind_addr {
        let peer = Peer::host(bind_addr, None).unwrap();
        PeerVariant::Tangled(peer)
    } else if let Some(state) = state {
        let peer = steam_networking::SteamPeer::new_host(
            lobbytype,
            state.client,
            250,
            LobbyExtraData {
                name: "no name specified".to_string(),
                game_mode: None,
            },
        );
        PeerVariant::Steam(peer)
    } else {
        println!("no steam");
        exit(1)
    };
    let player_path = netmaninit.player_path.clone();
    let netman = NetManager::new(variant, netmaninit, audio);
    netman.start_inner(player_path, Some(kind)).unwrap();
}
