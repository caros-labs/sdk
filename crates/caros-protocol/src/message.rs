use crate::{AppId, AppStatus, LifecycleEvent};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    NotFound,
    PermissionDenied,
    AlreadyActive,
    InvalidRequest,
    IncompatibleVersion,
    Unavailable,
    Internal,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
}
impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "params", rename_all = "snake_case")]
pub enum Request {
    ListApps,
    PlaybackStatus,
    OpenPlayback,
    SetPlaybackVolume {
        app: Option<AppId>,
        level: Option<u8>,
        muted: Option<bool>,
    },
    SetMasterVolume {
        volume: crate::playback::Volume,
    },
    MicrophoneStatus,
    OpenMicrophone,
    SetMicrophoneMuted {
        muted: bool,
    },
    AcquireService {
        name: String,
    },
    ReleaseService {
        name: String,
    },
    ServiceStatus {
        name: String,
    },
    AppStatus {
        id: AppId,
    },
    StartApp {
        id: AppId,
    },
    StopApp {
        id: AppId,
    },
    Subscribe,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ClientMessage {
    Hello { version: u32 },
    Request { id: u64, request: Request },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Reply {
    Apps(Vec<AppStatus>),
    PlaybackStatus(crate::playback::PlaybackStatus),
    PlaybackOpened,
    MicrophoneStatus(crate::microphone::MicrophoneStatus),
    MicrophoneOpened,
    Service(crate::ServiceStatus),
    Status(AppStatus),
    Accepted,
    /// Snapshot and sequence are captured atomically with subscription creation.
    Subscribed {
        apps: Vec<AppStatus>,
        sequence: u64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum ServerMessage {
    Hello {
        version: u32,
    },
    Rejected(ApiError),
    Response {
        id: u64,
        result: Result<Reply, ApiError>,
    },
    Event(LifecycleEvent),
    /// Connection closes afterwards; obtain a new snapshot by subscribing again.
    Lagged {
        missed: u64,
    },
}
