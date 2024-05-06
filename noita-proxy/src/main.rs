use eframe::NativeOptions;
use noita_proxy::App;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), eframe::Error> {
    let my_subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish();
    tracing::subscriber::set_global_default(my_subscriber).expect("setting tracing default failed");
    eframe::run_native(
        "Noita Proxy",
        NativeOptions::default(),
        Box::new(|_cc| Box::new(App::default())),
    )
}
