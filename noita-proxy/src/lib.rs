use std::{net::TcpListener, thread, time::Duration};

use eframe::egui;
use tungstenite::accept;

pub fn ws_server() {
    thread::spawn(|| {
        let server = TcpListener::bind("127.0.0.1:41251").unwrap();
        for stream in server.incoming() {
            let stream = stream.unwrap();
            stream.set_nodelay(true).ok();
            stream
                .set_read_timeout(Some(Duration::from_millis(1)))
                .expect("can set read timeout");
            println!("New stream incoming");

            let mut websocket = accept(stream).unwrap();
            println!("New stream connected");
            loop {
                let msg = websocket.read().unwrap();

                // We do not want to send back ping/pong messages.
                if msg.is_binary() || msg.is_text() {
                    websocket.send(msg).unwrap();
                }
            }
        }
    });
}

#[derive(Default)]
pub struct App {}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("hi");
        });
    }
}
