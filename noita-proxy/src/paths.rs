//! Paths that the proxy will uses

use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

pub const DEFAULT_PROXY_LOG_NAME: &str = "ew_log.txt";
pub const DEFAULT_OLD_PROXY_LOG_NAME: &str = "ew_log_old.txt";
pub const DEFAULT_SETTINGS_NAME: &str = "proxy.ron";
pub const DEFAULT_SAVE_STATE_NAME: &str = "save_state"; // this is a dir

pub const STEAM_COMPATDATA_NOITA_SAVE: &str =
    "compatdata/881100/pfx/drive_c/users/steamuser/AppData/LocalLow/Nolla_Games_Noita";

pub const NOITA_QUANTEW_INSTALL: &str = "mods/quant.ew";
pub const QUANTEW_PLAYER_SPRITESHEET: &str = "files/system/player/unmodified.png";

pub const PROJECT_DIRS_ORGANIZATION: &str = "quant";
pub const PROJECT_DIRS_APPLICATION: &str = "entangledworlds";

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct Paths {
    /// Usually
    /// - Windows: [TODO]
    /// - Linux: `/home/<name>/.local/share/Steam/steamapps/common/Noita`
    ///
    /// This is not serialized because it's currently always realized from [`Self::noita_exe`].
    #[serde(skip)]
    pub noita_install: Option<PathBuf>,

    /// Usually
    /// - Windows: [TODO]
    /// - Linux: `/home/<name>/.local/share/Steam/steamapps/common/Noita/noita.exe`
    pub noita_exe: Option<PathBuf>,

    /// Usually
    /// - Windows: [TODO]
    /// - Linux: `/home/<name>/.local/share/Steam/steamapps/common/Noita/mods/quant.ew`
    ///
    /// This is not serialized because it's currently always realized from [`Self::noita_exe`].
    #[serde(skip)]
    pub noita_quantew_install: Option<PathBuf>,

    /// Usually
    /// - Windows: [TODO]
    /// - Linux: `/home/<name>/.local/share/Steam/steamapps/common/Noita/mods/quant.ew/files/system/player/unmodified.png`
    ///
    /// This is not serialized because it's currently always realized from [`Self::noita_exe`].
    #[serde(skip)]
    pub noita_quantew_player_spritesheet: Option<PathBuf>,

    /// Usually
    /// - Windows: `C:\Users\<name>\AppData\LocalLow\Nolla_Games_Noita`
    /// - Linux: `/home/<name>/.local/share/Steam/steamapps/compatdata/881100/pfx/drive_c/users/steamuser/AppData/LocalLow/Nolla_Games_Noita`
    pub noita_save: Option<PathBuf>,

    /// Usually
    /// - Windows: [TODO]
    /// - Linux: `/home/<name>/.config/entangledworlds/proxy.ron`
    ///
    /// This is not serialized because this path is always either taken from CLI args or searched.
    #[serde(skip)]
    pub proxy_settings: Option<PathBuf>,

    /// Usually
    /// - Windows: [TODO]
    /// - Linux: `/home/<name>/.local/share/entangledworlds/save_state/`
    ///
    /// This is not serialized because this path is always either taken from CLI args or searched.
    #[serde(skip)]
    pub proxy_save_state: Option<PathBuf>,
}

#[rustfmt::skip]
impl Paths {
    pub fn noita_install(&self) -> &PathBuf {
        self.noita_install.as_ref().expect("noita_install path is Some")
    }
    pub fn noita_exe(&self) -> &PathBuf {
        self.noita_exe.as_ref().expect("noita_exe path is Some")
    }
    pub fn noita_quantew_install(&self) -> &PathBuf {
        self.noita_quantew_install.as_ref().expect("noita_quantew_install path is Some")
    }
    pub fn noita_quantew_player_spritesheet(&self) -> &PathBuf {
        self.noita_quantew_player_spritesheet.as_ref().expect("noita_quantew_player_spritesheet path is Some")
    }
    pub fn noita_save(&self) -> &PathBuf {
        self.noita_save.as_ref().expect("noita_save path is Some")
    }
    pub fn proxy_settings(&self) -> &PathBuf {
        self.proxy_settings.as_ref().expect("proxy_settings path is Some")
    }
    pub fn proxy_save_state(&self) -> &PathBuf {
        self.proxy_save_state.as_ref().expect("save_state path is Some")
    }
}

pub fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("", PROJECT_DIRS_ORGANIZATION, PROJECT_DIRS_APPLICATION)
}

pub fn proxy_exe_dir() -> PathBuf {
    std::env::current_exe()
        .map(|p| p.parent().unwrap().to_path_buf())
        .unwrap_or(".".into())
}

/// Set `noita_install` from `noita_exe`
/// Set `noita_quantew_install` from `noita_install`
/// Set `noita_quantew_player_spritesheet` from `noita_quantew_install`
pub fn realize_noita_paths_from_noita_exe(paths: &mut Paths) {
    let noita_exe = paths.noita_exe();
    paths.noita_install = Some(
        noita_exe
            .parent()
            .expect("noita_exe is valid")
            .to_path_buf(),
    );
    paths.noita_quantew_install = Some(paths.noita_install().join(NOITA_QUANTEW_INSTALL));
    paths.noita_quantew_player_spritesheet = Some(
        paths
            .noita_quantew_install()
            .join(QUANTEW_PLAYER_SPRITESHEET),
    );
}
