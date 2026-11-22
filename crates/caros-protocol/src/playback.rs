//! Fixed-format playback: 48 kHz stereo S16LE, 20 ms per packet.
use crate::AppId;
use serde::{Deserialize, Serialize};
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
pub const PLAYBACK_SAMPLES: usize = 1920;
pub const PLAYBACK_RATE: u32 = 48000;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Volume {
    pub level: u8,
    pub muted: bool,
}
impl Default for Volume {
    fn default() -> Self {
        Self {
            level: 100,
            muted: false,
        }
    }
}
impl Volume {
    pub fn gain(self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.level as f32 / 100.0
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppPlaybackStatus {
    pub app: AppId,
    pub volume: Volume,
    pub streams: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaybackStatus {
    pub master: Volume,
    pub apps: Vec<AppPlaybackStatus>,
    pub available: bool,
    /// Peak of the last mixed PCM block, before the OS audio server.
    pub mixed_peak: u16,
    pub mixed_frames: u64,
}
pub async fn write_frame(
    stream: &mut (impl AsyncWrite + Unpin),
    samples: &[i16],
) -> io::Result<()> {
    if samples.len() != PLAYBACK_SAMPLES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected 1920 interleaved stereo samples",
        ));
    }
    let mut bytes = Vec::with_capacity(PLAYBACK_SAMPLES * 2);
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    stream.write_u32(bytes.len() as u32).await?;
    stream.write_all(&bytes).await
}
pub async fn read_frame(stream: &mut (impl AsyncRead + Unpin)) -> io::Result<Vec<i16>> {
    if stream.read_u32().await? as usize != PLAYBACK_SAMPLES * 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid playback frame length",
        ));
    }
    let mut bytes = [0; PLAYBACK_SAMPLES * 2];
    stream.read_exact(&mut bytes).await?;
    Ok(bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| i16::from_le_bytes(*b))
        .collect())
}
