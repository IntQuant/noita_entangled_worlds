#![windows_subsystem = "windows"]

use std::{
    fs::File,
    io::{self, BufWriter},
    panic,
};

use eframe::{
    egui::{IconData, ViewportBuilder},
    NativeOptions,
};
use noita_proxy::{args::Args, connect_cli, host_cli, App};
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

#[allow(clippy::needless_return)]
#[tokio::main(worker_threads = 2)]
async fn main() {
    let log_to_file = cfg!(windows);
    let writer = {
        if log_to_file {
            let file = File::create("ew_log.txt");
            println!("Creating a log file");
            match file {
                Ok(file) => Box::new(BufWriter::new(file)) as Box<dyn io::Write + Send>,
                Err(_) => Box::new(io::stdout()) as Box<dyn io::Write + Send>,
            }
        } else {
            Box::new(io::stdout()) as Box<dyn io::Write + Send>
        }
    };

    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(writer);

    let my_subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_writer(non_blocking_writer)
        .finish();

    tracing::subscriber::set_global_default(my_subscriber).expect("setting tracing default failed");

    panic::set_hook(Box::new(|info| {
        let mut payload = "no panic payload available".to_string();

        if let Some(p) = info.payload().downcast_ref::<&str>() {
            payload = p.to_string();
        }

        if let Some(p) = info.payload().downcast_ref::<String>() {
            payload = p.to_string();
        }

        match info.location() {
            Some(loc) => {
                let file = loc.file();
                let line = loc.line();
                let col = loc.column();
                error!("Panic occured at {file}:{line}:{col} : {payload}");
            }
            None => {
                error!("Panic occured at unknown location: {payload}");
            }
        }
    }));

    let args: Args = argh::from_env();

    info!("Launch command: {:?}", args.launch_cmd);

    if let Some(host) = args.host {
        let port = if host.eq_ignore_ascii_case("steam") {
            0
        } else {
            host.parse::<u16>().unwrap_or(5123)
        };
        host_cli(port)
    } else if let Some(lobby) = args.lobby {
        connect_cli(lobby)
    } else {
        let icon = image::load_from_memory(include_bytes!("../assets/icon.png"))
            .unwrap()
            .to_rgba8();
        let icon = IconData {
            width: icon.width(),
            height: icon.height(),
            rgba: icon.into_vec(),
        };
        eframe::run_native(
            "Noita Proxy", // Don't change that, it defines where settings are stored.
            NativeOptions {
                viewport: ViewportBuilder::default()
                    .with_min_inner_size([800.0, 600.0])
                    .with_inner_size([1200.0, 800.0])
                    .with_icon(icon)
                    .with_title("Noita Entangled Worlds Proxy"),
                ..Default::default()
            },
            Box::new(|cc| Ok(Box::new(App::new(cc, args)))),
        )
        .unwrap()
    }
}
