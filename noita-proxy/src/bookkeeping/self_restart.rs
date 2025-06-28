use std::{
    convert::Infallible,
    io,
    process::{Command, exit},
};

use crate::lobby_code::{LobbyCode, LobbyKind};

pub struct SelfRestarter {
    command: Command,
}

impl SelfRestarter {
    pub fn new() -> io::Result<Self> {
        // Hopefully fine to restart ourselves?
        let exe_path = std::env::current_exe()?;
        let command = Command::new(exe_path);
        Ok(Self { command })
    }

    pub fn override_lobby_kind(&mut self, lobby_mode: LobbyKind) -> &mut Self {
        self.command.arg("--override-lobby-kind");
        self.command.arg(format!("{lobby_mode:?}"));
        self
    }

    pub fn connect_to(&mut self, lobby: LobbyCode) -> &mut Self {
        self.command.arg("--auto-connect-to");
        self.command.arg(lobby.serialize());
        self
    }

    pub fn restart(&mut self) -> io::Result<Infallible> {
        self.command.spawn()?;
        exit(0)
    }
}
