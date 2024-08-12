use std::{
    env, fs, io,
    net::{SocketAddr, TcpListener},
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

use bitcode::{Decode, Encode};
use socket2::{Domain, Socket, Type};
use steamworks::AppId;
use tracing::{info, warn};
use tungstenite::accept;

use crate::{
    bookkeeping::noita_launcher::{LaunchTokenResult, NoitaLauncher},
    steam_helper,
};

#[derive(Decode, Encode)]
struct RecorderEntry {
    timestamp_ms: u64,
    data: Vec<u8>,
}

pub(crate) struct Recorder {
    started: Instant,
    recording: Vec<RecorderEntry>,
    recording_dir: PathBuf,
    output_counter: u32,
    last_dump: Instant,
}

impl Default for Recorder {
    // This is a debug feature, so error handling can be lazier than usual.
    fn default() -> Self {
        let exe_path = std::env::current_exe().expect("path to exist");
        let exe_dir_path = exe_path.parent().unwrap();
        let recordings_base = exe_dir_path.join("crashcatcher_recordings");

        // Find the earliest free path to put recordings in.
        let recording_dir = (1u64..)
            .map(|num| recordings_base.join(format!("recording_{num:02}")))
            .skip_while(|path| path.try_exists().unwrap_or(true))
            .next()
            .expect("at least one name should be free");

        fs::create_dir_all(&recording_dir).expect("can create directory");

        info!("Recorder created. Will save to {}", recording_dir.display());

        Self {
            started: Instant::now(),
            recording: Vec::new(),
            recording_dir,
            output_counter: 0,
            last_dump: Instant::now(),
        }
    }
}

impl Recorder {
    pub(crate) fn record_msg(&mut self, msg: &tungstenite::Message) {
        match msg {
            tungstenite::Message::Binary(data) => {
                let elapsed = self.started.elapsed();
                self.recording.push(RecorderEntry {
                    timestamp_ms: elapsed.as_millis() as u64,
                    data: data.clone(),
                })
            }
            tungstenite::Message::Text(_) => {
                // These aren't used anyway.
            }
            // The rest isn't useful to save.
            _ => {}
        }
        if self.last_dump.elapsed() > Duration::from_secs(5) {
            self.dump_recording();
        }
    }

    fn dump_recording(&mut self) {
        let path = self
            .recording_dir
            .join(format!("rec_{:04}.bit", self.output_counter));
        info!("Dumping recording to {}", path.display());
        self.output_counter += 1;
        let data = bitcode::encode(&self.recording);
        self.recording.clear();
        thread::spawn(move || {
            let compressed_data = lz4_flex::compress_prepend_size(&data);
            info!("Compressed!");
            fs::write(path, compressed_data).expect("can write a file");
            info!("Done!");
        });
        self.last_dump = Instant::now();
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        if !self.recording.is_empty() {
            self.dump_recording()
        }
    }
}

fn recording_part_iter(recording_dir: PathBuf) -> impl Iterator<Item = Vec<RecorderEntry>> {
    (0..)
        .map(move |num| recording_dir.join(format!("rec_{:04}.bit", num)))
        .take_while(|p| p.exists())
        .map(|p| {
            let data = fs::read(p).unwrap();
            let data = lz4_flex::decompress_size_prepended(&data).unwrap();
            bitcode::decode(&data).unwrap()
        })
}

fn recorder_entry_iter(recording_dir: PathBuf) -> impl Iterator<Item = RecorderEntry> {
    recording_part_iter(recording_dir).flatten()
}

fn replay_loop(recording_dir: PathBuf) {
    let mut entry_iter = recorder_entry_iter(recording_dir).peekable();

    let socket = Socket::new(Domain::IPV4, Type::STREAM, None).unwrap();

    let address: SocketAddr = env::var("NP_NOITA_ADDR")
        .ok()
        .and_then(|x| x.parse().ok())
        .unwrap_or_else(|| "127.0.0.1:21251".parse().unwrap());

    info!("Listening for noita connection on {}", address);

    let address = address.into();
    socket.bind(&address).unwrap();
    socket.listen(1).unwrap();
    socket.set_nonblocking(false).unwrap();

    let local_server: TcpListener = socket.into();

    let Ok((stream, addr)) = local_server.accept() else {
        panic!("Could not start server");
    };

    info!("New stream incoming from {}", addr);
    stream.set_nodelay(true).ok();
    stream.set_nonblocking(false).ok();

    let mut ws = accept(stream).unwrap();

    let stream_ref = ws.get_ref();
    stream_ref.set_nonblocking(true).ok();
    stream_ref
        .set_read_timeout(Some(Duration::from_millis(1)))
        .expect("can set read timeout");

    info!("Websocket connection accepted");

    let started_at = Instant::now();
    while let Some(next_entry) = entry_iter.peek() {
        loop {
            let msg = ws.read();

            match msg {
                Ok(_msg) => {}
                Err(tungstenite::Error::Io(io_err))
                    if io_err.kind() == io::ErrorKind::WouldBlock
                        || io_err.kind() == io::ErrorKind::TimedOut =>
                {
                    break
                }
                Err(err) => {
                    warn!("Error occured while reading from websocket: {}", err);
                    return;
                }
            }
        }
        if (next_entry.timestamp_ms as u128) < started_at.elapsed().as_millis() {
            let next_entry = entry_iter.next().unwrap();
            ws.write(tungstenite::Message::Binary(next_entry.data))
                .unwrap();
            ws.flush().unwrap();
        } else {
            thread::sleep(Duration::from_millis(1));
        }
    }

    info!("Server stopped");
}

pub fn replay_file(path: PathBuf) {
    info!("Will replay {}", path.display());
    let mut steam_state = steam_helper::SteamState::new().expect("Can init steam state");

    let apps = steam_state.client.apps();
    let app_id = AppId::from(881100);
    let app_install_dir = apps.app_install_dir(app_id);
    let game_exe_path = PathBuf::from(app_install_dir).join("noita.exe");

    let mut launcher = NoitaLauncher::new(&game_exe_path, None, Some(&mut steam_state));
    let LaunchTokenResult::Ok(mut token) = launcher.launch_token() else {
        panic!("Expected to be able to launch the game");
    };

    info!("Ready to start the game");

    let loop_thread = thread::spawn(|| replay_loop(path));

    token.start_game();

    loop_thread.join().unwrap();
}
