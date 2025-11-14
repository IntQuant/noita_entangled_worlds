#![windows_subsystem = "windows"]

use eframe::{
    NativeOptions,
    egui::{IconData, ViewportBuilder},
};
use noita_proxy::{App, args::Args, connect_cli, host_cli};
use std::{
    backtrace, fs,
    fs::File,
    io::{self, BufWriter},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    panic,
};
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

#[allow(clippy::needless_return)]
#[tokio::main(worker_threads = 2)]
async fn main() {
    #[cfg(target_os = "windows")]
    {
        use winapi::um::wincon::{ATTACH_PARENT_PROCESS, AttachConsole};
        unsafe {
            AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }
    let log = if let Ok(path) = std::env::current_exe() {
        path.parent().unwrap().join("ew_log.txt")
    } else {
        "ew_log.txt".into()
    };
    if log.exists() {
        let _ = fs::copy(
            log.clone(),
            log.clone().parent().unwrap().join("ew_log_old.txt"),
        );
    }
    let file = File::create(log);
    println!("Creating a log file");
    let writer = match file {
        Ok(file) => Box::new(BufWriter::new(file)) as Box<dyn io::Write + Send>,
        Err(_) => Box::new(io::stdout()) as Box<dyn io::Write + Send>,
    };

    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(writer);

    let my_subscriber = tracing_subscriber::FmtSubscriber::builder().with_env_filter(
        EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy(),
    );
    if cfg!(debug_assertions) {
        let my_subscriber = my_subscriber.finish();
        tracing::subscriber::set_global_default(my_subscriber)
            .expect("setting tracing default failed");
    } else {
        let my_subscriber = my_subscriber
            .with_writer(non_blocking_writer)
            .with_ansi(false)
            .finish();
        tracing::subscriber::set_global_default(my_subscriber)
            .expect("setting tracing default failed");
    }

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

        let trace = backtrace::Backtrace::force_capture();
        error!("Backtrace: {}", trace);
    }));

    let args: Args = argh::from_env();

    info!("Launch command: {:?}", args.launch_cmd);

    if let Some(host) = args.clone().host {
        let bind_addr = if host.eq_ignore_ascii_case("steam") {
            None
        } else {
            // allows binding to both IPv6 and IPv4
            host.parse::<SocketAddr>()
                .ok()
                // compatibility with providing only the port (which then proceeds to bind to IPv4 only)
                .or_else(|| {
                    Some(SocketAddr::V4(SocketAddrV4::new(
                        Ipv4Addr::UNSPECIFIED,
                        host.parse().ok()?,
                    )))
                })
                .map(Some)
                .expect("host argument is neither SocketAddr nor port")
        };
        host_cli(bind_addr, args)
    } else if let Some(lobby) = args.clone().lobby {
        connect_cli(lobby, args)
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
