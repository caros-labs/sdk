use crate::{Error, ManagementClient, session::Session};
pub use caros_protocol::microphone::{MicrophonePacket, MicrophoneStatus};
use caros_protocol::{Reply, Request};
use tokio::{net::UnixStream, time::timeout};

/// All device access occurs in Core. Applications only receive PCM over IPC.
pub struct Microphone<'a> {
    client: &'a ManagementClient,
}
impl<'a> Microphone<'a> {
    pub(crate) fn new(client: &'a ManagementClient) -> Self {
        Self { client }
    }
    pub async fn status(&self) -> Result<MicrophoneStatus, Error> {
        match self.client.request(Request::MicrophoneStatus).await? {
            Reply::MicrophoneStatus(status) => Ok(status),
            _ => Err(Error::Protocol("expected microphone status".into())),
        }
    }
    /// Opens a dedicated binary stream. Requires an administrator microphone grant
    /// and the calling process to be a running Core-supervised application.
    pub async fn open(&self) -> Result<MicrophoneStream, Error> {
        timeout(self.client.timeout, async {
            let mut session = Session::connect(&self.client.path).await?;
            session.request(Request::OpenMicrophone).await?;
            Ok(MicrophoneStream {
                socket: Some(session.stream),
            })
        })
        .await
        .map_err(|_| Error::Timeout)?
    }
    /// Requires a separate microphone control grant.
    pub async fn set_muted(&self, muted: bool) -> Result<(), Error> {
        self.client
            .request(Request::SetMicrophoneMuted { muted })
            .await?;
        Ok(())
    }
}
/// Dropping or closing releases the subscription. Read errors (including cancelled
/// reads) require reopening: never resume a partially read binary packet.
pub struct MicrophoneStream {
    socket: Option<UnixStream>,
}
impl MicrophoneStream {
    pub async fn read(&mut self) -> Result<MicrophonePacket, Error> {
        let mut socket = self.socket.take().ok_or(Error::Disconnected)?;
        let packet = MicrophonePacket::read(&mut socket).await?;
        self.socket = Some(socket);
        Ok(packet)
    }
    pub fn close(self) {
        drop(self);
    }
}
