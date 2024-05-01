use eframe::NativeOptions;
use noita_proxy::App;

fn main() -> Result<(), eframe::Error> {
    let my_subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(my_subscriber).expect("setting tracing default failed");
    eframe::run_native(
        "Noita Proxy",
        NativeOptions::default(),
        Box::new(|_cc| Box::new(App::default())),
    )
}
