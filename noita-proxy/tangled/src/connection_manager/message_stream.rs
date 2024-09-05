use std::io;

use quinn::{RecvStream, SendStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::DirectConnectionError;

pub(crate) struct SendMessageStream {
    inner: SendStream,
}

pub(crate) struct RecvMessageStream {
    inner: RecvStream,
}

impl SendMessageStream {
    pub(crate) fn new(inner: SendStream) -> Self {
        Self { inner }
    }

    pub(crate) async fn send(&mut self, msg: &[u8]) -> Result<(), DirectConnectionError> {
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
}

impl RecvMessageStream {
    pub(crate) fn new(inner: RecvStream) -> Self {
        Self { inner }
    }

    pub(crate) async fn recv(&mut self) -> Result<Vec<u8>, DirectConnectionError> {
        let len = self
            .inner
            .read_u32()
            .await
            .map_err(|_err| DirectConnectionError::MessageIoFailed)?;
        let mut buf = vec![0; len as usize];
        self.inner
            .read_exact(&mut buf)
            .await
            .map_err(|_err| DirectConnectionError::MessageIoFailed)?;
        Ok(buf)
    }
}
