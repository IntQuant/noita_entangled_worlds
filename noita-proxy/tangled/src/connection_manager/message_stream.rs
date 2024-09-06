use std::marker::PhantomData;

use bitcode::{DecodeOwned, Encode};
use quinn::{RecvStream, SendStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::trace;

use super::DirectConnectionError;

pub(crate) struct SendMessageStream<Msg> {
    inner: SendStream,
    _phantom: PhantomData<fn(Msg)>,
}

pub(crate) struct RecvMessageStream<Msg> {
    inner: RecvStream,
    _phantom: PhantomData<fn() -> Msg>,
}

impl<Msg: Encode> SendMessageStream<Msg> {
    pub(crate) fn new(inner: SendStream) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    async fn send_raw(&mut self, msg: &[u8]) -> Result<(), DirectConnectionError> {
        self.inner
            .write_u32(
                msg.len()
                    .try_into()
                    .expect("Only messages up to ~4GB supported"),
            )
            .await
            .map_err(|_err| DirectConnectionError::MessageIoFailed)?;
        self.inner
            .write_all(msg)
            .await
            .map_err(|_err| DirectConnectionError::MessageIoFailed)?;
        Ok(())
    }

    pub(crate) async fn send(&mut self, msg: &Msg) -> Result<(), DirectConnectionError> {
        let msg = bitcode::encode(msg);
        self.send_raw(&msg).await
    }
}

impl<Msg: DecodeOwned> RecvMessageStream<Msg> {
    pub(crate) fn new(inner: RecvStream) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    async fn recv_raw(&mut self) -> Result<Vec<u8>, DirectConnectionError> {
        let len = self
            .inner
            .read_u32()
            .await
            .map_err(|_err| DirectConnectionError::MessageIoFailed)?;
        trace!("Expecting message of len {len}");
        let mut buf = vec![0; len as usize];
        self.inner
            .read_exact(&mut buf)
            .await
            .map_err(|_err| DirectConnectionError::MessageIoFailed)?;
        Ok(buf)
    }
    pub(crate) async fn recv(&mut self) -> Result<Msg, DirectConnectionError> {
        let raw = self.recv_raw().await?;
        bitcode::decode(&raw).map_err(|_| DirectConnectionError::DecodeError)
    }
}
