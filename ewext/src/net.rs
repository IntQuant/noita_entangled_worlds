use std::{env, net::SocketAddr};

use shared::{NoitaInbound, NoitaOutbound, message_socket::MessageSocket};

pub(crate) struct NetManager {
    socket: MessageSocket<NoitaInbound, NoitaOutbound>,
}

impl NetManager {
    pub(crate) fn new() -> eyre::Result<Self> {
        let address: SocketAddr = env::var("NP_NOITA_ADDR")
            .ok()
            .and_then(|x| x.parse().ok())
            .unwrap_or_else(|| SocketAddr::new("127.0.0.1".parse().unwrap(), 21251));
        println!("Connecting to {address:?}");
        let socket = MessageSocket::connect(&address)?;

        Ok(NetManager { socket })
    }

    pub(crate) fn send(&mut self, msg: &NoitaOutbound) -> eyre::Result<()> {
        self.socket.write(msg)
    }

    pub(crate) fn recv(&mut self) -> eyre::Result<NoitaInbound> {
        self.socket.read()
    }

    pub(crate) fn try_recv(&mut self) -> eyre::Result<Option<NoitaInbound>> {
        self.socket.try_read()
    }

    pub(crate) fn flush(&mut self) -> eyre::Result<()> {
        self.socket.flush()
    }
}
