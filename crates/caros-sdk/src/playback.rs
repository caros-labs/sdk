use crate::{Error, ManagementClient, session::Session};
use caros_protocol::{AppId, Reply, Request, playback};
pub use playback::{AppPlaybackStatus, PLAYBACK_RATE, PLAYBACK_SAMPLES, PlaybackStatus, Volume};
use tokio::{
    io::AsyncReadExt,
    net::UnixStream,
    time::{Duration, timeout},
};
pub struct Playback<'a> {
    client: &'a ManagementClient,
}
impl<'a> Playback<'a> {
    pub(crate) fn new(client: &'a ManagementClient) -> Self {
        Self { client }
    }
    pub async fn status(&self) -> Result<PlaybackStatus, Error> {
        match self.client.request(Request::PlaybackStatus).await? {
            Reply::PlaybackStatus(status) => Ok(status),
            _ => Err(Error::Protocol("expected playback status".into())),
        }
    }
    pub async fn open(&self) -> Result<PlaybackStream, Error> {
        timeout(self.client.timeout, async {
            let mut session = Session::connect(&self.client.path).await?;
            session.request(Request::OpenPlayback).await?;
            Ok(PlaybackStream {
                socket: Some(session.stream),
            })
        })
        .await
        .map_err(|_| Error::Timeout)?
    }
    pub async fn set_volume(&self, level: u8) -> Result<(), Error> {
        self.change(None, Some(level), None).await
    }
    pub async fn set_muted(&self, muted: bool) -> Result<(), Error> {
        self.change(None, None, Some(muted)).await
    }
    /// Requires a playback control grant, unless target is the caller itself.
    pub async fn set_app_volume(&self, app: AppId, volume: Volume) -> Result<(), Error> {
        self.change(Some(app), Some(volume.level), Some(volume.muted))
            .await
    }
    pub async fn set_master(&self, volume: Volume) -> Result<(), Error> {
        self.client
            .request(Request::SetMasterVolume { volume })
            .await?;
        Ok(())
    }
    async fn change(
        &self,
        app: Option<AppId>,
        level: Option<u8>,
        muted: Option<bool>,
    ) -> Result<(), Error> {
        self.client
            .request(Request::SetPlaybackVolume { app, level, muted })
            .await?;
        Ok(())
    }
}
pub struct PlaybackStream {
    socket: Option<UnixStream>,
}
impl PlaybackStream {
    /// Sends one 20 ms stereo block. Core acknowledgement means queued, not audible.
    /// Backpressure bounds queueing. Cancellation/error invalidates this stream.
    pub async fn write(&mut self, samples: &[i16]) -> Result<(), Error> {
        if samples.len() != PLAYBACK_SAMPLES {
            return Err(Error::Protocol("expected 1920 samples".into()));
        }
        let mut socket = self.socket.take().ok_or(Error::Disconnected)?;
        timeout(Duration::from_secs(2), async {
            playback::write_frame(&mut socket, samples).await?;
            if socket.read_u8().await? != 1 {
                return Err(Error::Protocol("invalid playback acknowledgement".into()));
            }
            Ok::<_, Error>(())
        })
        .await
        .map_err(|_| Error::Timeout)??;
        self.socket = Some(socket);
        Ok(())
    }
    /// Stops this stream and discards pending frames; does not drain the device.
    pub fn close(self) {
        drop(self);
    }
}
