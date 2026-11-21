//! Versioned local management API. Process configuration stays inside Core.
mod app;
mod id;
mod message;
pub mod microphone;
mod service;
pub mod wire;

pub use app::*;
pub use id::AppId;
pub use message::*;
pub use service::*;

pub const PROTOCOL_VERSION: u32 = 2;
