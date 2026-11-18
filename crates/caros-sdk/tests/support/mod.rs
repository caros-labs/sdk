use caros_protocol::{AppId, ClientMessage, PROTOCOL_VERSION, ServerMessage, wire};
use std::{fs, path::PathBuf};
use tokio::net::{UnixListener, UnixStream};

pub struct Endpoint {
    pub path: PathBuf,
    pub listener: UnixListener,
}
impl Endpoint {
    pub fn new() -> Self {
        let path = std::env::temp_dir().join(format!("caros-sdk-{}.sock", AppId::new()));
        let listener = UnixListener::bind(&path).unwrap();
        Self { path, listener }
    }
    pub async fn accept(&self) -> UnixStream {
        let (mut stream, _) = self.listener.accept().await.unwrap();
        assert!(matches!(
            wire::read::<ClientMessage>(&mut stream).await.unwrap(),
            ClientMessage::Hello {
                version: PROTOCOL_VERSION
            }
        ));
        wire::write(
            &mut stream,
            &ServerMessage::Hello {
                version: PROTOCOL_VERSION,
            },
        )
        .await
        .unwrap();
        stream
    }
}
impl Drop for Endpoint {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
