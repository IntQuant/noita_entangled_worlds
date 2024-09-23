use eframe::{
    egui::{IconData, ViewportBuilder},
    NativeOptions,
};
use noita_proxy::{args::Args, recorder::replay_file, App};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

#[tokio::main(worker_threads = 2)]
async fn main() -> Result<(), eframe::Error> {
    let my_subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish();
    tracing::subscriber::set_global_default(my_subscriber).expect("setting tracing default failed");

    let args: Args = argh::from_env();

    info!("{:?}", args.launch_cmd);

    if let Some(replay) = args.replay_folder {
        replay_file(replay);
        return Ok(());
    }

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
                .with_icon(icon)
                .with_title("Noita Entangled Worlds Proxy"),
            follow_system_theme: false,
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc, args)))),
    )
}
