use crate::{Apps, Error, Subscription, session::Session};
use caros_protocol::{Reply, Request};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{sync::Mutex, time::timeout};

/// Serializes control requests. No automatic replay: a timed-out start/stop may
/// already have executed on Core. Reconnect and query status before retrying.
pub struct ManagementClient {
    pub(crate) path: PathBuf,
    pub(crate) timeout: Duration,
    session: Mutex<Option<Session>>,
}
impl ManagementClient {
    pub async fn connect(path: impl AsRef<Path>) -> Result<Self, Error> {
        Self::connect_with_timeout(path, Duration::from_secs(5)).await
    }
    pub async fn connect_with_timeout(
        path: impl AsRef<Path>,
        duration: Duration,
    ) -> Result<Self, Error> {
        let path = path.as_ref().to_path_buf();
        let session = timeout(duration, Session::connect(&path))
            .await
            .map_err(|_| Error::Timeout)??;
        Ok(Self {
            path,
            timeout: duration,
            session: Mutex::new(Some(session)),
        })
    }
    pub fn playback(&self) -> crate::Playback<'_> {
        crate::Playback::new(self)
    }
    pub fn microphone(&self) -> crate::Microphone<'_> {
        crate::Microphone::new(self)
    }
    pub fn services(&self) -> crate::Services<'_> {
        crate::Services::new(self)
    }
    pub fn apps(&self) -> Apps<'_> {
        Apps::new(self)
    }

    /// A dedicated stream prevents event traffic from blocking management replies.
    pub async fn subscribe(&self) -> Result<Subscription, Error> {
        timeout(self.timeout, Subscription::connect(&self.path))
            .await
            .map_err(|_| Error::Timeout)?
    }

    pub(crate) async fn request(&self, request: Request) -> Result<Reply, Error> {
        timeout(self.timeout, async {
            let mut slot = self.session.lock().await;
            // Taking ownership makes cancellation safe: partially used streams
            // are dropped rather than put back into the connection pool.
            let mut session = slot.take().ok_or(Error::Disconnected)?;
            let result = session.request(request).await;
            if result.is_ok() || matches!(result, Err(Error::Remote(_))) {
                *slot = Some(session);
            }
            result
        })
        .await
        .map_err(|_| Error::Timeout)?
    }
}
