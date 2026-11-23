use caros_protocol::ApiError;
use std::{fmt, io};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Timeout,
    Disconnected,
    Protocol(String),
    Remote(ApiError),
    Lagged { missed: u64 },
}
impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::Timeout => write!(f, "Core request timed out; reconnect before retrying"),
            Self::Disconnected => write!(f, "Core connection is closed"),
            Self::Protocol(message) => write!(f, "Core protocol error: {message}"),
            Self::Remote(error) => write!(f, "Core {:?}: {}", error.code, error.message),
            Self::Lagged { missed } => write!(
                f,
                "missed {missed} lifecycle events; resubscribe for a fresh snapshot"
            ),
        }
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}
