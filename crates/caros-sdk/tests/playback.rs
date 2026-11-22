mod support;
use caros_protocol::{ClientMessage, Reply, Request, ServerMessage, playback, wire};
use caros_sdk::{Error, ManagementClient, PLAYBACK_SAMPLES};
use tokio::{
    io::AsyncWriteExt,
    time::{Duration, timeout},
};
#[tokio::test]
async fn playback_uses_dedicated_binary_stream_and_cancellation_invalidates_it() {
    let endpoint = support::Endpoint::new();
    let path = endpoint.path.clone();
    let (received, wait) = tokio::sync::oneshot::channel();
    let (finish, finished) = tokio::sync::oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        let _management = endpoint.accept().await;
        let mut audio = endpoint.accept().await;
        let ClientMessage::Request {
            id,
            request: Request::OpenPlayback,
        } = wire::read(&mut audio).await.unwrap()
        else {
            panic!()
        };
        wire::write(
            &mut audio,
            &ServerMessage::Response {
                id,
                result: Ok(Reply::PlaybackOpened),
            },
        )
        .await
        .unwrap();
        assert_eq!(
            playback::read_frame(&mut audio).await.unwrap(),
            vec![123; PLAYBACK_SAMPLES]
        );
        audio.write_u8(1).await.unwrap();
        playback::read_frame(&mut audio).await.unwrap();
        received.send(()).unwrap();
        let _ = finished.await; // deliberately withhold the second acknowledgement
    });
    let client = ManagementClient::connect(path).await.unwrap();
    let mut stream = client.playback().open().await.unwrap();
    stream.write(&vec![123; PLAYBACK_SAMPLES]).await.unwrap();
    assert!(
        timeout(
            Duration::from_millis(50),
            stream.write(&vec![0; PLAYBACK_SAMPLES])
        )
        .await
        .is_err()
    );
    wait.await.unwrap();
    assert!(matches!(
        stream.write(&vec![0; PLAYBACK_SAMPLES]).await,
        Err(Error::Disconnected)
    ));
    finish.send(()).unwrap();
    task.await.unwrap();
}
