use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::{Child, Command},
};

struct NoitaStartCmd {
    executable: OsString,
    args: Vec<OsString>,
}

impl NoitaStartCmd {
    fn from_full_args(args: &[OsString]) -> Option<Self> {
        let (executable, args) = args.split_first()?;
        Some(Self {
            executable: executable.to_owned(),
            args: args.to_vec(),
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
    // game_exe_path: PathBuf,
    start_args: Option<NoitaStartCmd>,
    noita_process: Option<Child>,
}

impl NoitaLauncher {
    pub fn new(game_exe_path: &Path, start_args: Option<&str>) -> Self {
        let game_dir_path = game_exe_path
            .parent()
            .expect("game directory to exist")
            .to_path_buf();

        let default_start_args = if cfg!(target_os = "windows") {
            let executable = game_exe_path.as_os_str().to_owned();
            Some(NoitaStartCmd {
                executable,
                args: Vec::new(),
            })
        } else {
            None
        };

        let start_args = start_args
            .and_then(shlex::split)
            .map(|v| v.into_iter().map(OsString::from).collect::<Vec<_>>())
            .and_then(|args| NoitaStartCmd::from_full_args(&args));
        let start_args = start_args.or(default_start_args);

        Self {
            game_dir_path,
            start_args,
            noita_process: None,
        }
    }

    fn is_noita_running(&mut self) -> bool {
        match self.noita_process.as_mut().map(|child| child.try_wait()) {
            Some(Ok(Some(_))) => false, // Already existed
            Some(Ok(None)) => true,     // Not yet exited
            Some(Err(_)) => false,      // Could not wait for child.
            None => false,              // No child stored.
        }
    }

    pub fn launch_token(&mut self) -> LaunchTokenResult {
        if self.is_noita_running() {
            return LaunchTokenResult::AlreadyStarted;
        }

        if self.start_args.is_some() {
            LaunchTokenResult::Ok(LaunchToken(self))
        } else {
            LaunchTokenResult::CantStart
        }
    }
}

impl LaunchToken<'_> {
    pub fn start_game(&mut self) {
        let start_cmd = self.0.start_args.as_ref().unwrap();
        let child = Command::new(&start_cmd.executable)
            .current_dir(&self.0.game_dir_path)
            .args(&start_cmd.args)
            .spawn();
        self.0.noita_process = child.ok();
    }
}
