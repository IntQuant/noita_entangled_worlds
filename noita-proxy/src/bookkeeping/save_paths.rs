use std::fs;
use std::path::PathBuf;

use tracing::{error, info, warn};

use crate::{bookkeeping::settings::Settings, paths};

/// Paths that are saved and loaded by the proxy
pub(crate) struct SavePaths {
    pub settings_path: PathBuf,
    pub save_state_path: PathBuf,
}

impl SavePaths {
    pub fn new(settings_path: PathBuf, save_state_path: PathBuf) -> Self {
        SavePaths {
            settings_path,
            save_state_path,
        }
    }

    pub fn new_with_maybe_override(
        settings_path: Option<PathBuf>,
        save_state_path: Option<PathBuf>,
    ) -> SavePaths {
        use Prefer::*;
        enum Prefer {
            Custom,
            NextToExe,
            ProjectDirs,
        }

        let project_dirs = paths::project_dirs();
        let proxy_exe_dir = paths::proxy_exe_dir();
        let settings_next_to_exe_path = proxy_exe_dir.join(paths::DEFAULT_SETTINGS_NAME);

        let settings_prefer: Prefer;
        let settings_path = if let Some(settings_path) = settings_path {
            settings_prefer = Custom;
            settings_path
        } else if settings_next_to_exe_path.exists() {
            settings_prefer = NextToExe;
            settings_next_to_exe_path
        } else if let Some(project_dirs) = &project_dirs {
            settings_prefer = ProjectDirs;
            project_dirs.config_dir().join(paths::DEFAULT_SETTINGS_NAME)
        } else {
            warn!(
                "There is no path override and failed to get project dirs. Falling back to 'next to exe' to store settings and save states."
            );
            settings_prefer = NextToExe;
            settings_next_to_exe_path
        };

        let save_state_path = if let Some(save_state_path) = save_state_path {
            save_state_path
        } else {
            let get_project_dirs_path = || {
                project_dirs
                    .as_ref()
                    .expect("project_dirs is already checked to be some")
                    .data_dir()
                    .join(paths::DEFAULT_SAVE_STATE_NAME)
            };
            match settings_prefer {
                Custom if project_dirs.is_some() => get_project_dirs_path(),
                Custom => proxy_exe_dir.join(paths::DEFAULT_SAVE_STATE_NAME),
                ProjectDirs => get_project_dirs_path(),
                NextToExe => proxy_exe_dir.join(paths::DEFAULT_SAVE_STATE_NAME),
            }
        };

        if matches!(settings_prefer, ProjectDirs) {
            let _ = fs::create_dir_all(
                project_dirs
                    .expect("project_dirs is already checked to be some")
                    .config_dir(),
            );
        }
        if let Some(save_state_path_parent) = &save_state_path.parent() {
            let _ = fs::create_dir_all(save_state_path_parent);
        }

        info!("Settings path: {}", settings_path.display());
        info!("Save state path: {}", save_state_path.display());

        Self::new(settings_path, save_state_path)
    }

    pub fn load_settings(&self) -> Settings {
        if let Ok(mut file) = File::open(&self.settings_path) {
            let mut s = String::new();
            let _ = file.read_to_string(&mut s);
            ron::from_str::<Settings>(&s).unwrap_or_default()
        } else {
            info!("Failed to load settings file, returing default settings");
            Settings::default()
        }
    }

    pub fn save_settings(&self, settings: Settings) {
        let settings = ron::to_string(&settings).unwrap();
        if let Ok(mut file) = File::create(&self.settings_path) {
            file.write_all(settings.as_bytes()).unwrap();
        }
    }
}
