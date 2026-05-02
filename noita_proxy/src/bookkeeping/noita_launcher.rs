use std::{
    ffi::OsString,
    fs::File,
    io::{BufRead, BufReader},
    mem,
    path::{Path, PathBuf},
    process::{Child, Command},
};

use steamworks::AppId;
use tracing::{info, warn};

use crate::{paths::Paths, steam_helper::SteamState};

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
    noita_install: PathBuf,
    start_args: Option<NoitaStartCmd>,
    noita_process: Option<Child>,
}

impl NoitaLauncher {
    pub fn new(
        paths: &Paths,
        start_args: Option<&str>,
        run_with_gdb: bool,
        steam_state: Option<&mut SteamState>,
    ) -> Self {
        let noita_install = paths.noita_install();

        let default_start_args = || {
            if cfg!(target_os = "windows") {
                Some(NoitaStartCmd {
                    executable: paths.noita_exe().as_os_str().to_owned(),
                    args: Vec::new(),
                    steam_install: None,
                    noita_compat_data: None,
                    noita_install: None,
                })
            } else {
                linux_try_get_noita_start_cmd(paths, steam_state)
            }
        };

        let start_args = start_args
            .and_then(shlex::split)
            .map(|v| v.into_iter().map(OsString::from).collect::<Vec<_>>())
            .and_then(|args| NoitaStartCmd::from_full_args(&args));
        let mut start_args = start_args.or_else(default_start_args);

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
            noita_install: noita_install.to_path_buf(),
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
    paths: &Paths,
    steam_state: Option<&mut SteamState>,
) -> Option<NoitaStartCmd> {
    let steam_install = paths.steam_install().to_owned();
    let noita_compatdata_path = steam_install.join("steamapps/compatdata/881100/");
    if !noita_compatdata_path.exists() {
        return None;
    }
    let config_info_path = noita_compatdata_path.join("config_info");
    let config_info_file = File::open(config_info_path)
        .inspect_err(|err| warn!("Couldn't open config_info : {}", err))
        .ok()?;
    let proton_fonts = BufReader::new(config_info_file)
        .lines()
        .nth(1)?
        .inspect_err(|err| warn!("Couldn't find proton fonts paths: {}", err))
        .ok()?;
    let proton_fonts = Path::new(&proton_fonts);
    let proton_install = proton_path_from_fonts(proton_fonts)?;
    let proton_binary = proton_install.join("proton").into_os_string();
    let tool_manifest = File::open(proton_install.join("toolmanifest.vdf"))
        .inspect_err(|err| warn!("Couldn't open toolmanifest.vdf file: {}", err))
        .ok()?;
    let runtime_appid = BufReader::new(tool_manifest)
        .lines()
        .map(|l| l.unwrap())
        .find(|l| l.contains("require_tool_appid"))
        .map(|a| a.split('"').nth(3).map(|b| b.parse::<u32>()));

    let noita_exe = paths.noita_exe().as_os_str().to_owned();
    let noita_install = paths.noita_install().to_owned();
    let (executable, args) = match (steam_state, runtime_appid) {
        (Some(state), Some(Some(Ok(1628350)))) => {
            let apps = state.client.apps();
            let app_id = AppId::from(1628350);
            let app_install_dir = apps.app_install_dir(app_id);
            (
                PathBuf::from(app_install_dir)
                    .join("_v2-entry-point")
                    .into(),
                vec!["--verb=run".into(), proton_binary, "run".into(), noita_exe],
            )
        }
        (_, Some(Some(Ok(_)))) => {
            let app_install_dir = proton_install.parent()?.join("SteamLinuxRuntime_sniper");
            if app_install_dir.exists() {
                (
                    app_install_dir.join("_v2-entry-point").into(),
                    vec!["--verb=run".into(), proton_binary, "run".into(), noita_exe],
                )
            } else {
                (proton_binary, vec!["run".into(), noita_exe])
            }
        }
        _ => (proton_binary, vec!["run".into(), noita_exe]),
    };
    Some(NoitaStartCmd {
        executable,
        args,
        steam_install: Some(steam_install),
        noita_compat_data: Some(noita_compatdata_path),
        noita_install: Some(noita_install),
    })
}

fn proton_path_from_fonts(proton_path_fonts: &Path) -> Option<PathBuf> {
    Some(proton_path_fonts.parent()?.parent()?.parent()?.into())
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
                .current_dir(&self.0.noita_install)
                .args(&start_cmd.args)
                .spawn()
        } else {
            Command::new(&start_cmd.executable)
                .env("NP_NOITA_ADDR", &addr_env)
                .env("SteamAppId", "881100")
                .env("SteamGameId", "881100")
                .current_dir(&self.0.noita_install)
                .args(&start_cmd.args)
                .spawn()
        };
        self.0.noita_process = child.ok();
    }
}
