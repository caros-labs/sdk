use crate::{Error, session::Session};
use caros_protocol::{AppStatus, LifecycleEvent, Reply, Request, ServerMessage, wire};
use std::path::Path;
use tokio::net::UnixStream;

pub struct Subscription {
    snapshot: Vec<AppStatus>,
    sequence: u64,
    stream: Option<UnixStream>,
}
impl Subscription {
    pub(crate) async fn connect(path: &Path) -> Result<Self, Error> {
        let mut session = Session::connect(path).await?;
        match session.request(Request::Subscribe).await? {
            Reply::Subscribed { apps, sequence } => Ok(Self {
                snapshot: apps,
                sequence,
                stream: Some(session.stream),
            }),
            _ => Err(Error::Protocol("missing subscription snapshot".into())),
        }
    }
    /// State at subscription establishment; subsequent events do not mutate it.
    pub fn snapshot(&self) -> &[AppStatus] {
        &self.snapshot
    }
    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Cancelling a pending read closes this subscription. Open a new one to
    /// obtain a consistent snapshot rather than silently skip partial frames.
    pub async fn next(&mut self) -> Result<LifecycleEvent, Error> {
        let mut stream = self.stream.take().ok_or(Error::Disconnected)?;
        match wire::read(&mut stream).await? {
            ServerMessage::Event(event) if Some(event.sequence) == self.sequence.checked_add(1) => {
                self.sequence = event.sequence;
                self.stream = Some(stream);
                Ok(event)
            }
            ServerMessage::Lagged { missed } => Err(Error::Lagged { missed }),
            _ => Err(Error::Protocol(
                "unexpected lifecycle event or sequence gap".into(),
            )),
        }
    }
}
