use std::{env, sync::Arc, thread, time::Duration};

use eframe::egui::{self, RichText, TextureHandle, Ui};
use steamworks::{LobbyId, SResult, SteamAPIInitError, SteamId};
use tokio::time;
use tracing::{error, info, warn};

use crate::{GameMode, releases::Version};

pub struct SteamUserAvatar {
    avatar: TextureHandle,
}

impl SteamUserAvatar {
    pub fn display_with_labels(&self, ui: &mut Ui, label_top: &str, label_bottom: &str) {
        let image = egui::Image::new(&self.avatar).fit_to_exact_size([32.0, 32.0].into());
        ui.scope(|ui| {
            ui.set_min_width(200.0);
            ui.horizontal(|ui| {
                ui.add(image);
                ui.vertical(|ui| {
                    ui.label(RichText::new(label_top).size(14.0));
                    ui.label(RichText::new(label_bottom).size(11.0));
                });
            });
        });
    }
}

pub(crate) enum MaybeLobbyList {
    Pending,
    List(Arc<Vec<LobbyId>>),
    Errored,
}

enum LobbyListState {
    None,
    Pending {
        list: Arc<Vec<LobbyId>>,
        receiver: tokio::sync::oneshot::Receiver<SResult<Vec<LobbyId>>>,
    },
    List {
        list: Arc<Vec<LobbyId>>,
    },
    Errored,
}

impl LobbyListState {
    fn is_none(&self) -> bool {
        matches!(self, LobbyListState::None)
    }
    fn try_resolve(&mut self) -> MaybeLobbyList {
        match self {
            LobbyListState::None => MaybeLobbyList::Pending,
            LobbyListState::Pending { list, receiver } => {
                let r = if list.is_empty() {
                    MaybeLobbyList::Pending
                } else {
                    MaybeLobbyList::List(list.clone())
                };
                if let Ok(lst) = receiver.try_recv() {
                    match lst {
                        Ok(list) => *self = LobbyListState::List { list: list.into() },
                        Err(err) => {
                            warn!("Failed to get lobby list: {:?}", err);
                            *self = LobbyListState::Errored
                        }
                    }
                }
                r
            }
            LobbyListState::List { list } => MaybeLobbyList::List(list.clone()),
            LobbyListState::Errored => MaybeLobbyList::Errored,
        }
    }
}

pub struct LobbyInfo {
    pub member_count: usize,
    pub member_limit: usize,
    pub version: Option<Version>,
    pub data: LobbyExtraData,
    pub is_noita_online: bool,
}

pub struct SteamState {
    pub client: steamworks::Client,
    lobby_state: LobbyListState,
}

pub struct LobbyExtraData {
    pub name: String,
    pub game_mode: Option<GameMode>,
}

impl SteamState {
    pub(crate) fn new(spacewars: bool) -> Result<Self, SteamAPIInitError> {
        if env::var_os("NP_DISABLE_STEAM").is_some() {
            return Err(SteamAPIInitError::FailedGeneric(
                "Disabled by env variable".to_string(),
            ));
        }
        let app_id = env::var("NP_APPID").ok().and_then(|x| x.parse().ok());
        info!("Initializing steam client...");
        let client =
            steamworks::Client::init_app(app_id.unwrap_or(if spacewars { 480 } else { 881100 }))?;
        info!("Initializing relay network accesss...");
        client.networking_utils().init_relay_network_access();
        if let Err(err) = client.networking_sockets().init_authentication() {
            error!("Failed to init_authentication: {}", err)
        }

        let single = client.clone();
        thread::spawn(move || {
            info!("Spawned steam callback thread");
            loop {
                single.run_callbacks();
                thread::sleep(Duration::from_millis(3));
            }
        });

        Ok(SteamState {
            client,
            lobby_state: LobbyListState::None,
        })
    }

    pub fn get_user_name(&self, id: SteamId) -> String {
        let friends = self.client.friends();
        friends.get_friend(id).name()
    }

    pub(crate) fn get_my_id(&self) -> SteamId {
        self.client.user().steam_id()
    }

    pub(crate) fn update_lobby_list(&mut self, timer: &mut time::Instant) {
        *timer = time::Instant::now();
        let (s, r) = tokio::sync::oneshot::channel();
        let matchmaking = self.client.matchmaking();
        matchmaking.set_request_lobby_list_distance_filter(steamworks::DistanceFilter::Worldwide);
        matchmaking.request_lobby_list(|res| {
            let _ = s.send(res);
        });
        let list = match &self.lobby_state {
            LobbyListState::List { list } => list.clone(),
            LobbyListState::Pending { list, receiver: _ } => list.clone(),
            _ => Arc::new(Vec::new()),
        };
        self.lobby_state = LobbyListState::Pending { list, receiver: r }
    }

    pub(crate) fn list_lobbies(&mut self, timer: &mut time::Instant) -> MaybeLobbyList {
        if self.lobby_state.is_none() || timer.elapsed().as_secs() > 8 {
            self.update_lobby_list(timer);
        }
        self.lobby_state.try_resolve()
    }

    pub(crate) fn lobby_info(&self, lobby: LobbyId) -> LobbyInfo {
        let matchmaking = self.client.matchmaking();
        let version = matchmaking
            .lobby_data(lobby, "ew_version")
            .and_then(Version::parse_from_diplay);
        let is_noita_online = matchmaking
            .lobby_data(lobby, "System")
            .map(|s| s.starts_with("NoitaOnline"))
            .unwrap_or(false);
        LobbyInfo {
            member_count: matchmaking.lobby_member_count(lobby),
            member_limit: matchmaking.lobby_member_limit(lobby).unwrap_or(250),
            version,
            data: LobbyExtraData {
                name: matchmaking
                    .lobby_data(lobby, "name")
                    .unwrap_or_default()
                    .to_owned(),
                game_mode: matchmaking
                    .lobby_data(lobby, "game_mode")
                    .and_then(|s| s.parse().ok()),
            },
            is_noita_online,
        }
    }
}
