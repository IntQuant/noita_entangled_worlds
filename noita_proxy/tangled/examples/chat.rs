use std::{
    env::args,
    io::stdin,
    thread::{sleep, spawn},
    time::Duration,
};

use crossbeam::channel::bounded;
use tangled::{Peer, Reliability};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let mut args = args().skip(1);
    let peer = match args.next().as_deref() {
        Some("host") => {
            let bind_addr = match args.next().and_then(|arg| arg.parse().ok()) {
                Some(addr) => addr,
                None => {
                    println!("Expected an address:port to host on as a second argument");
                    return;
                }
            };
            Peer::host(bind_addr, None)
        }
        Some("connect") => {
            let connect_addr = match args.next().and_then(|arg| arg.parse().ok()) {
                Some(addr) => addr,
                None => {
                    println!("Expected an address:port to connect to as a second argument");
                    return;
                }
            };
            Peer::connect(connect_addr, None)
        }
        Some(_) | None => {
            println!("First argument should be one of 'host', 'connect'");
            return;
        }
    }
    .unwrap();
    let (s, r) = bounded(1);
    spawn(move || {
        for msg in stdin().lines() {
            s.send(msg.unwrap()).unwrap();
        }
    });
    loop {
        for msg in peer.recv() {
            match msg {
                tangled::NetworkEvent::PeerConnected(id) => println!("Peer connected: {}", id),
                tangled::NetworkEvent::PeerDisconnected(id) => {
                    println!("Peer disconnected: {}", id)
                }
                tangled::NetworkEvent::Message(msg) => {
                    println!("{}", String::from_utf8_lossy(&msg.data))
                }
            }
        }
        for msg in r.try_iter() {
            println!("State: {:?}", peer.state());
            let data = msg.as_bytes();
            for destination in peer.iter_peer_ids() {
                if destination == peer.my_id().unwrap() {
                    continue;
                }
                println!("Sent to {}", destination);
                peer.send(destination, data.to_vec(), Reliability::Reliable)
                    .unwrap();
            }
        }
        sleep(Duration::from_millis(10));
    }
}
