use crate::Error;
use caros_protocol::{ClientMessage, PROTOCOL_VERSION, Reply, Request, ServerMessage, wire};
use std::path::Path;
use tokio::net::UnixStream;

pub(crate) struct Session {
    pub stream: UnixStream,
    next_id: u64,
}
impl Session {
    pub async fn connect(path: &Path) -> Result<Self, Error> {
        let mut stream = UnixStream::connect(path).await?;
        wire::write(
            &mut stream,
            &ClientMessage::Hello {
                version: PROTOCOL_VERSION,
            },
        )
        .await?;
        match wire::read(&mut stream).await? {
            ServerMessage::Hello { version } if version == PROTOCOL_VERSION => {
                Ok(Self { stream, next_id: 1 })
            }
            ServerMessage::Rejected(error) => Err(Error::Remote(error)),
            _ => Err(Error::Protocol("invalid version handshake".into())),
        }
    }
    pub async fn request(&mut self, request: Request) -> Result<Reply, Error> {
        let id = self.next_id;
        self.next_id = id
            .checked_add(1)
            .ok_or_else(|| Error::Protocol("request ID exhausted".into()))?;
        wire::write(
            &mut self.stream,
            &ClientMessage::Request {
                id,
                request: request.clone(),
            },
        )
        .await?;
        match wire::read(&mut self.stream).await? {
            ServerMessage::Response {
                id: received,
                result,
            } if received == id => {
                let reply = result.map_err(Error::Remote)?;
                if response_matches(&request, &reply) {
                    Ok(reply)
                } else {
                    Err(Error::Protocol(
                        "response payload does not match request".into(),
                    ))
                }
            }
            _ => Err(Error::Protocol("unexpected response or request ID".into())),
        }
    }
}

fn response_matches(request: &Request, reply: &Reply) -> bool {
    match (request, reply) {
        (Request::ListApps, Reply::Apps(_)) => true,
        (Request::PlaybackStatus, Reply::PlaybackStatus(_)) => true,
        (Request::OpenPlayback, Reply::PlaybackOpened) => true,
        (Request::SetPlaybackVolume { .. } | Request::SetMasterVolume { .. }, Reply::Accepted) => {
            true
        }
        (Request::MicrophoneStatus, Reply::MicrophoneStatus(_)) => true,
        (Request::OpenMicrophone, Reply::MicrophoneOpened) => true,
        (Request::SetMicrophoneMuted { .. }, Reply::Accepted) => true,
        (Request::AppStatus { id }, Reply::Status(status)) => *id == status.id,
        (Request::StartApp { .. } | Request::StopApp { .. }, Reply::Accepted) => true,
        (Request::Subscribe, Reply::Subscribed { .. }) => true,
        (
            Request::AcquireService { name } | Request::ServiceStatus { name },
            Reply::Service(status),
        ) => *name == status.name,
        (Request::ReleaseService { .. }, Reply::Accepted) => true,
        _ => false,
    }
}
