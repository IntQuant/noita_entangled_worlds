use crate::steam_helper::SteamState;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::{Child, Command},
};
use steamworks::AppId;
use tracing::{info, warn};
struct NoitaStartCmd {
    executable: OsString,
    args: Vec<OsString>,
    steam_install: Option<PathBuf>,
    noita_compat_data: Option<PathBuf>,
    noita_install: Option<PathBuf>,
}

impl NoitaStartCmd {
    fn from_full_args(args: &[OsString]) -> Option<Self> {
        let (executable, args) = args.split_first()?;
        Some(Self {
            executable: executable.to_owned(),
            args: args.to_vec(),
            steam_install: None,
            noita_compat_data: None,
            noita_install: None,
        })
    }
}

pub enum LaunchTokenResult<'a> {
    Ok(LaunchToken<'a>),
    AlreadyStarted,
    CantStart,
}

pub struct LaunchToken<'a>(&'a mut NoitaLauncher);

pub struct NoitaLauncher {
    game_dir_path: PathBuf,
    start_args: Option<NoitaStartCmd>,
    noita_process: Option<Child>,
}

impl NoitaLauncher {
    pub fn new(
        game_exe_path: &Path,
        start_args: Option<&str>,
        run_with_gdb: bool,
        steam_state: Option<&mut SteamState>,
    ) -> Self {
        let game_dir_path = game_exe_path
            .parent()
            .expect("game directory to exist")
            .to_path_buf();

        let default_start_args = if cfg!(target_os = "windows") {
            let executable = game_exe_path.as_os_str().to_owned();
            Some(NoitaStartCmd {
                executable,
                args: Vec::new(),
                steam_install: None,
                noita_compat_data: None,
                noita_install: None,
            })
        } else {
            linux_try_get_noita_start_cmd(game_exe_path, steam_state)
        };

        let start_args = start_args
            .and_then(shlex::split)
            .map(|v| v.into_iter().map(OsString::from).collect::<Vec<_>>())
            .and_then(|args| NoitaStartCmd::from_full_args(&args));
        let mut start_args = start_args.or(default_start_args);

        if let Some(start_args) = start_args.as_mut()
            && run_with_gdb
        {
            info!("Extending start cmd to run gdbserver");
            start_args.args.insert(
                0,
                mem::replace(&mut start_args.executable, "gdbserver".into()),
            );
            start_args.args.insert(0, "--".into());
            start_args.args.insert(0, "localhost:4123".into());
        }

        Self {
            game_dir_path,
            start_args,
            noita_process: None,
        }
    }

    fn check_if_noita_running(&mut self) -> bool {
        match self.noita_process.as_mut().map(|child| child.try_wait()) {
            Some(Ok(Some(_))) => false, // Already exited
            Some(Ok(None)) => true,     // Not yet exited
            Some(Err(_)) => false,      // Could not wait for child.
            None => false,              // No child stored.
        }
    }

    pub fn launch_token(&mut self) -> LaunchTokenResult<'_> {
        if self.check_if_noita_running() {
            return LaunchTokenResult::AlreadyStarted;
        }

        if self.start_args.is_some() {
            LaunchTokenResult::Ok(LaunchToken(self))
        } else {
            LaunchTokenResult::CantStart
        }
    }
}

fn linux_try_get_noita_start_cmd(
    game_exe_path: &Path,
    steam_state: Option<&mut SteamState>,
) -> Option<NoitaStartCmd> {
    let executable = game_exe_path.as_os_str().to_owned();
    // ~/.local/share/Steam/steamapps/common/Noita/noita.exe
    let game_path = game_exe_path.parent()?;
    let steamapps_path = game_path.parent()?.parent()?;
    let noita_compatdata_path = steamapps_path.join("compatdata/881100/");
    if noita_compatdata_path.exists() {
        let config_info_path = noita_compatdata_path.join("config_info");
        let config_info_file = File::open(config_info_path)
            .inspect_err(|err| warn!("Couldn't open proton fonts file: {}", err))
            .ok()?;
        let file = BufReader::new(config_info_file)
            .lines()
            .nth(1)?
            .inspect_err(|err| warn!("Couldn't find proton fonts paths: {}", err))
            .ok()?;
        let proton_path_fonts = Path::new(&file);
        let proton_path = proton_path_from_fonts(proton_path_fonts)?;
        let tool_manifest = File::open(proton_path.join("toolmanifest.vdf"))
            .inspect_err(|err| warn!("Couldn't open toolmanifest.vdf file: {}", err))
            .ok()?;
        let runtime_appid = BufReader::new(tool_manifest)
            .lines()
            .map(|l| l.unwrap())
            .find(|l| l.contains("require_tool_appid"))
            .map(|a| a.split('"').nth(3).map(|b| b.parse::<u32>()));
        match (steam_state, runtime_appid) {
            (Some(state), Some(Some(Ok(1628350)))) => {
                let apps = state.client.apps();
                let app_id = AppId::from(1628350);
                let app_install_dir = apps.app_install_dir(app_id);
                Some(NoitaStartCmd {
                    executable: PathBuf::from(app_install_dir)
                        .join("_v2-entry-point")
                        .into(),
                    args: vec![
                        "--verb=run".into(),
                        proton_path.join("proton").into_os_string(),
                        "run".into(),
                        executable,
                    ],
                    steam_install: steam_intall_path(steamapps_path),
                    noita_compat_data: Some(noita_compatdata_path),
                    noita_install: Some(game_path.to_path_buf()),
                })
            }
            (_, Some(Some(Ok(_)))) => {
                let app_install_dir = proton_path.parent()?.join("SteamLinuxRuntime_sniper");
                if app_install_dir.exists() {
                    Some(NoitaStartCmd {
                        executable: app_install_dir.join("_v2-entry-point").into(),
                        args: vec![
                            "--verb=run".into(),
                            proton_path.join("proton").into_os_string(),
                            "run".into(),
                            executable,
                        ],
                        steam_install: steam_intall_path(steamapps_path),
                        noita_compat_data: Some(noita_compatdata_path),
                        noita_install: Some(game_path.to_path_buf()),
                    })
                } else {
                    Some(NoitaStartCmd {
                        executable: proton_path.join("proton").into_os_string(),
                        args: vec!["run".into(), executable],
                        steam_install: steam_intall_path(steamapps_path),
                        noita_compat_data: Some(noita_compatdata_path),
                        noita_install: Some(game_path.to_path_buf()),
                    })
                }
            }
            _ => Some(NoitaStartCmd {
                executable: proton_path.join("proton").into_os_string(),
                args: vec!["run".into(), executable],
                steam_install: steam_intall_path(steamapps_path),
                noita_compat_data: Some(noita_compatdata_path),
                noita_install: Some(game_path.to_path_buf()),
            }),
        }
    } else {
        None
    }
}

fn proton_path_from_fonts(proton_path_fonts: &Path) -> Option<PathBuf> {
    Some(proton_path_fonts.parent()?.parent()?.parent()?.into())
}

fn steam_intall_path(steamapps_path: &Path) -> Option<PathBuf> {
    steamapps_path.parent().map(|p| p.to_path_buf())
}

impl LaunchToken<'_> {
    pub fn start_game(&mut self, port: u16) {
        let addr_env = format!("127.0.0.1:{port}");
        let start_cmd = self.0.start_args.as_ref().unwrap();
        let child = if let Some(game_path) = &start_cmd.noita_install {
            let steam_install = start_cmd.steam_install.clone().unwrap();
            let compat_data = start_cmd.noita_compat_data.clone().unwrap();
            std::env::set_current_dir(game_path).unwrap();

            info!("Steam install: {}", steam_install.display());
            info!("Compat data: {}", compat_data.display());
            info!("Game path: {}", game_path.display());
            info!("Exe path: {}", start_cmd.executable.to_str().unwrap());
            info!("Args: {:?}", start_cmd.args);

            Command::new(&start_cmd.executable)
                .env("STEAM_COMPAT_CLIENT_INSTALL_PATH", steam_install)
                .env("STEAM_COMPAT_DATA_PATH", compat_data)
                .env("NP_NOITA_ADDR", &addr_env)
                .env("SteamAppId", "881100")
                .env("SteamGameId", "881100")
                .current_dir(&self.0.game_dir_path)
                .args(&start_cmd.args)
                .spawn()
        } else {
            Command::new(&start_cmd.executable)
                .env("NP_NOITA_ADDR", &addr_env)
                .env("SteamAppId", "881100")
                .env("SteamGameId", "881100")
                .current_dir(&self.0.game_dir_path)
                .args(&start_cmd.args)
                .spawn()
        };
        self.0.noita_process = child.ok();
    }
}
