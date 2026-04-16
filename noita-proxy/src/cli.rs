use std::{net::SocketAddr, path::PathBuf, process::exit, thread::sleep, time::Duration};

use argh::{FromArgValue, FromArgs};
use tangled::Peer;

use crate::{
    AudioSettings,
    bookkeeping::{save_paths::SavePaths, save_state::SaveState, settings::Settings},
    game_settings::GameSettings,
    lobby_code::{LobbyCode, LobbyKind},
    mod_manager,
    net::{NetManager, NetManagerInit, NetManagerPaths, omni::PeerVariant, steam_networking},
    paths,
    player_cosmetics::PlayerPngDesc,
    steam_helper,
    util::steam_helper::LobbyExtraData,
};

#[derive(FromArgs, PartialEq, Debug, Clone)]
/// Noita proxy.
pub struct Args {
    /// noita launch command that will be used.
    #[argh(option)]
    pub launch_cmd: Option<String>,
    /// adjust ui scale; default is 1.0.
    #[argh(option)]
    pub ui_zoom_factor: Option<f32>,
    /// steam lobby code.
    #[argh(option)]
    pub lobby: Option<String>,
    /// host either steam or ip.
    #[argh(option)]
    pub host: Option<String>,
    /// noita.exe path
    #[argh(option)]
    pub exe_path: Option<PathBuf>,
    /// language for gui
    #[argh(option)]
    pub language: Option<String>,

    /// overrides the default settings file location
    #[argh(option)]
    pub settings_path: Option<PathBuf>,

    /// overrides the default save state file location
    #[argh(option)]
    pub save_state_path: Option<PathBuf>,

    // Used internally.
    /// override lobby mode to use. Options: "Gog", "Steam".
    #[argh(option)]
    pub override_lobby_kind: Option<LobbyKind>,
    /// used internally.
    #[argh(option)]
    pub auto_connect_to: Option<LobbyCode>,

    /// also run gdbserver when starting noita. Used for development.
    #[argh(switch)]
    pub run_noita_with_gdb: bool,
}

impl FromArgValue for LobbyKind {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        match value {
            "Steam" => Ok(LobbyKind::Steam),
            "Gog" => Ok(LobbyKind::Gog),
            _ => Err("Unknown mode".to_string()),
        }
    }
}

impl FromArgValue for LobbyCode {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        LobbyCode::parse(value).map_err(|e| e.to_string())
    }
}

fn cli_setup(
    args: Args,
) -> (
    Option<steam_helper::SteamState>,
    NetManagerInit,
    LobbyKind,
    AudioSettings,
    steamworks::LobbyType,
    GameSettings,
) {
    let save_paths = SavePaths::new_with_maybe_override(
        args.settings_path.clone(),
        args.save_state_path.clone(),
    );
    let settings = save_paths.load_settings();
    let Settings {
        color: appearance,
        app: saved_state,
        audio,
        mut paths,
    } = settings;
    paths.proxy_settings = Some(save_paths.settings_path.clone());
    paths.proxy_save_state = Some(save_paths.save_state_path.clone());
    let mut state = steam_helper::SteamState::new(saved_state.spacewars).ok();
    let my_nickname = saved_state
        .nickname
        .unwrap_or("no nickname found".to_string());

    if let Some(state) = &mut state {
        mod_manager::try_find_game_path(&mut paths, Some(state));
    } else if let Some(p) = args.exe_path {
        paths.noita_exe = Some(p);
    } else {
        panic!("noita.exe is not provided and can't find it in settings.");
    }
    paths::realize_noita_paths_from_noita_exe(&mut paths);
    mod_manager::try_find_save_path(&mut paths);
    let run_save_state = SaveState::new(save_paths.save_state_path);
    let mut cosmetics = (false, false, false);
    if let Some(path) = &paths.noita_save {
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
    let paths =
        NetManagerPaths::try_from_paths(&paths).expect("necessary paths for networking are some");
    let netmaninit = NetManagerInit {
        my_nickname,
        save_state: run_save_state,
        cosmetics,
        paths,
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
        saved_state.game_settings,
    )
}

pub fn connect_cli(lobby: String, args: Args) {
    let (state, netmaninit, kind, audio, _, _) = cli_setup(args);
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
    let player_path = netmaninit.paths.noita_quantew_player_spritesheet.clone();
    let netman = NetManager::new(variant, netmaninit, audio);
    netman.start_inner(player_path, Some(kind)).unwrap();
}

/// Bind to the provided `bind_addr` with `args` with CLI output only.
///
/// The `bind_addr` is either `Some` address/port pair to bind to, or `None` to use Steam networking.
pub fn host_cli(bind_addr: Option<SocketAddr>, args: Args) {
    let (state, netmaninit, kind, audio, lobbytype, game_settings) = cli_setup(args);
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
    let player_path = netmaninit.paths.noita_quantew_player_spritesheet.clone();
    let netman = NetManager::new(variant, netmaninit, audio);
    *netman.settings.lock().unwrap() = game_settings;
    netman.start_inner(player_path, Some(kind)).unwrap();
}
