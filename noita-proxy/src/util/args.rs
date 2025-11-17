use std::path::PathBuf;

use argh::{FromArgValue, FromArgs};

use crate::lobby_code::{LobbyCode, LobbyKind};

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
