//! Trusted local management client. This API is for Shell and developer tools,
//! not a permission boundary between apps running under the same Unix user.
mod apps;
mod client;
mod error;
mod microphone;
mod playback;
pub use playback::{
    AppPlaybackStatus, PLAYBACK_RATE, PLAYBACK_SAMPLES, Playback, PlaybackStatus, PlaybackStream,
    Volume,
};
mod services;
pub use microphone::{Microphone, MicrophonePacket, MicrophoneStatus, MicrophoneStream};
mod session;
mod subscription;

pub use apps::Apps;
pub use caros_protocol::ServiceStatus;
pub use caros_protocol::{
    ApiError, AppId, AppState, AppStatus, ErrorCode, EventKind, ExitInfo, LifecycleEvent,
};
pub use client::ManagementClient;
pub use error::Error;
pub use services::Services;
pub use subscription::Subscription;
