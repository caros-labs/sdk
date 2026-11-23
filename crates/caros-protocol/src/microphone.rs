//! Microphone v1: 48 kHz, mono, signed 16-bit little-endian PCM, 20 ms frames.
use crate::AppId;
use serde::{Deserialize, Serialize};
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const MICROPHONE_RATE: u32 = 48_000;
pub const MICROPHONE_SAMPLES: usize = 960;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MicrophoneStatus {
    pub sample_rate: u32,
    pub channels: u8,
    pub muted: bool,
    pub consumers: Vec<AppId>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum MicrophonePacket {
    /// Sequence increases across dropped frames; timestamp is monotonic microseconds
    /// since this capture worker started. Concurrent subscriptions share the timeline;
    /// a worker started after the final consumer exits begins a new timeline.
    Frame {
        sequence: u64,
        timestamp_us: u64,
        samples: Vec<i16>,
    },
    Gap {
        missed: u64,
    },
    Unavailable,
    Ready,
}
impl MicrophonePacket {
    pub async fn write(&self, stream: &mut (impl AsyncWrite + Unpin)) -> io::Result<()> {
        let mut data = Vec::with_capacity(1940);
        match self {
            Self::Frame {
                sequence,
                timestamp_us,
                samples,
            } => {
                if samples.len() != MICROPHONE_SAMPLES {
                    return Err(invalid());
                }
                data.push(0);
                data.extend_from_slice(&sequence.to_le_bytes());
                data.extend_from_slice(&timestamp_us.to_le_bytes());
                for sample in samples {
                    data.extend_from_slice(&sample.to_le_bytes());
                }
            }
            Self::Gap { missed } => {
                data.push(1);
                data.extend_from_slice(&missed.to_le_bytes());
            }
            Self::Unavailable => data.push(2),
            Self::Ready => data.push(3),
        }
        stream.write_u32(data.len() as u32).await?;
        stream.write_all(&data).await
    }
    pub async fn read(stream: &mut (impl AsyncRead + Unpin)) -> io::Result<Self> {
        let len = stream.read_u32().await? as usize;
        if !(1..=17 + MICROPHONE_SAMPLES * 2).contains(&len) {
            return Err(invalid());
        }
        let mut data = vec![0; len];
        stream.read_exact(&mut data).await?;
        Ok(match (data[0], len) {
            (0, n) if n == 17 + MICROPHONE_SAMPLES * 2 => Self::Frame {
                sequence: u64::from_le_bytes(data[1..9].try_into().unwrap()),
                timestamp_us: u64::from_le_bytes(data[9..17].try_into().unwrap()),
                samples: data[17..]
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|s| i16::from_le_bytes([s[0], s[1]]))
                    .collect(),
            },
            (1, 9) => Self::Gap {
                missed: u64::from_le_bytes(data[1..9].try_into().unwrap()),
            },
            (2, 1) => Self::Unavailable,
            (3, 1) => Self::Ready,
            _ => return Err(invalid()),
        })
    }
}
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid microphone packet")
}
