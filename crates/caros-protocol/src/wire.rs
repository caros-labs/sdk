//! Four-byte big-endian length followed by UTF-8 JSON. Oversized frames are
//! rejected before allocating payload storage. Partial reads are not cancel-safe:
//! discard the connection if an operation times out or is cancelled.
use serde::{Serialize, de::DeserializeOwned};
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const MAX_FRAME_BYTES: usize = 256 * 1024;

pub async fn read<T: DeserializeOwned>(reader: &mut (impl AsyncRead + Unpin)) -> io::Result<T> {
    let size = reader.read_u32().await? as usize;
    if size == 0 || size > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid frame length",
        ));
    }
    let mut bytes = vec![0; size];
    reader.read_exact(&mut bytes).await?;
    serde_json::from_slice(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

pub async fn write<T: Serialize>(
    writer: &mut (impl AsyncWrite + Unpin),
    value: &T,
) -> io::Result<()> {
    let bytes = serde_json::to_vec(value).map_err(io::Error::other)?;
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame exceeds size limit",
        ));
    }
    writer.write_u32(bytes.len() as u32).await?;
    writer.write_all(&bytes).await?;
    writer.flush().await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn fragmented_frame_round_trips() {
        let (mut tx, mut rx) = tokio::io::duplex(1);
        let sending = tokio::spawn(async move {
            write(&mut tx, &vec![1, 2, 3]).await.unwrap();
        });
        assert_eq!(read::<Vec<u8>>(&mut rx).await.unwrap(), vec![1, 2, 3]);
        sending.await.unwrap();
    }
    #[tokio::test]
    async fn rejects_invalid_lengths_json_and_truncation() {
        for bytes in [
            0u32.to_be_bytes().to_vec(),
            (MAX_FRAME_BYTES as u32 + 1).to_be_bytes().to_vec(),
            vec![0, 0, 0, 1, b'{'],
            vec![0, 0, 0, 5, b'a'],
        ] {
            assert!(
                read::<serde_json::Value>(&mut bytes.as_slice())
                    .await
                    .is_err()
            );
        }
    }
    #[tokio::test]
    async fn rejects_oversized_output() {
        let mut sink = tokio::io::sink();
        assert!(
            write(&mut sink, &"x".repeat(MAX_FRAME_BYTES))
                .await
                .is_err()
        );
    }
}
