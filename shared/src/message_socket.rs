use std::{
    io::{BufReader, BufWriter, Read, Write},
    marker::PhantomData,
    net::{SocketAddr, TcpStream},
    sync::mpsc::{self, RecvError, TryRecvError},
    thread::{self, JoinHandle},
    time::Duration,
};

use bitcode::{DecodeOwned, Encode};
use eyre::{bail, Context};
use tracing::info;

fn read_one<T: DecodeOwned>(mut buf: impl Read) -> eyre::Result<T> {
    let mut len_buf = [0u8; 4];
    buf.read_exact(&mut len_buf)
        .wrap_err("Couldn't receive the length from stream")?;
    let len = u32::from_le_bytes(len_buf);
    let mut out_buf = vec![0; usize::try_from(len)?];
    buf.read_exact(out_buf.as_mut_slice())
        .wrap_err("Couldn't read message body")?;
    bitcode::decode(&out_buf).wrap_err("Failed to decode message body")
}

pub struct MessageSocket<Inbound, Outbound> {
    socket: BufWriter<TcpStream>,
    recv_messages: mpsc::Receiver<eyre::Result<Inbound>>,
    reader_thread: Option<JoinHandle<()>>,
    _phantom: PhantomData<fn() -> Outbound>,
}

impl<Inbound: DecodeOwned + Send + 'static, Outbound: Encode> MessageSocket<Inbound, Outbound> {
    pub fn new(socket: TcpStream) -> eyre::Result<Self> {
        socket.set_write_timeout(Some(Duration::from_secs(10)))?;
        let (sender, recv_messages) = mpsc::channel();
        let reader_thread = Some(thread::spawn({
            let socket = socket.try_clone()?;
            move || {
                let mut socket = BufReader::new(socket);
                loop {
                    let res = read_one(&mut socket);
                    let res_was_error = res.is_err();
                    if sender.send(res).is_err() {
                        break;
                    }
                    if res_was_error {
                        break;
                    }
                }
            }
        }));

        Ok(Self {
            socket: BufWriter::new(socket),
            recv_messages,
            reader_thread,
            _phantom: PhantomData,
        })
    }

    pub fn connect(addr: &SocketAddr) -> eyre::Result<Self> {
        let stream = TcpStream::connect_timeout(addr, Duration::from_secs(1))?;
        Self::new(stream).wrap_err("Failed to wrap socket")
    }

    pub fn read(&mut self) -> eyre::Result<Inbound> {
        match self.recv_messages.recv() {
            Ok(msg) => msg,
            Err(RecvError) => bail!("Channel disconnected"),
        }
    }

    pub fn try_read(&mut self) -> eyre::Result<Option<Inbound>> {
        match self.recv_messages.try_recv() {
            Ok(msg) => Some(msg).transpose(),
            Err(TryRecvError::Disconnected) => bail!("Channel disconnected"),
            Err(TryRecvError::Empty) => Ok(None),
        }
    }

    // Surely doing a blocking write won't be a problem over a loopback interface.
    pub fn write(&mut self, value: &Outbound) -> eyre::Result<()> {
        let encoded = bitcode::encode(value);
        self.socket
            .write_all(&u32::to_le_bytes(
                u32::try_from(encoded.len()).wrap_err("Message too large to be sent")?,
            ))
            .wrap_err("Couldn't write length to stream")?;
        self.socket
            .write_all(&encoded)
            .wrap_err("Couldn't write message body to stream")?;
        Ok(())
    }

    pub fn flush(&mut self) -> eyre::Result<()> {
        self.socket.flush()?;
        Ok(())
    }
}

impl<Inbound, Outbound> Drop for MessageSocket<Inbound, Outbound> {
    fn drop(&mut self) {
        self.socket
            .get_mut()
            .shutdown(std::net::Shutdown::Both)
            .ok();
        if let Some(handle) = self.reader_thread.take() {
            handle.join().ok();
        }
        info!("Message socket dropped");
    }
}
