use eframe::NativeOptions;
use noita_proxy::{ws_server, App};

fn main() -> Result<(), eframe::Error> {
    ws_server();
    eframe::run_native(
        "Noita Proxy",
        NativeOptions::default(),
        Box::new(|_cc| Box::new(App::default())),
    )
}
