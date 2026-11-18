mod support;
use caros_protocol::{ApiError, ClientMessage, ErrorCode, Reply, ServerMessage, wire};
use caros_sdk::{Error, ManagementClient};
use std::time::Duration;
use support::Endpoint;
use tokio::{io::AsyncReadExt, sync::oneshot, time::timeout};

#[tokio::test]
async fn mismatched_response_id_closes_client() {
    let endpoint = Endpoint::new();
    let path = endpoint.path.clone();
    let server = tokio::spawn(async move {
        let mut stream = endpoint.accept().await;
        let _: ClientMessage = wire::read(&mut stream).await.unwrap();
        wire::write(
            &mut stream,
            &ServerMessage::Response {
                id: 99,
                result: Ok(Reply::Apps(vec![])),
            },
        )
        .await
        .unwrap();
    });
    let client = ManagementClient::connect(&path).await.unwrap();
    assert!(matches!(
        client.apps().list().await,
        Err(Error::Protocol(_))
    ));
    assert!(matches!(
        client.apps().list().await,
        Err(Error::Disconnected)
    ));
    server.await.unwrap();
}

#[tokio::test]
async fn request_timeout_discards_stream_without_replaying_command() {
    let endpoint = Endpoint::new();
    let path = endpoint.path.clone();
    let server = tokio::spawn(async move {
        let mut stream = endpoint.accept().await;
        let _: ClientMessage = wire::read(&mut stream).await.unwrap();
        let mut byte = [0];
        assert_eq!(
            timeout(Duration::from_secs(2), stream.read(&mut byte))
                .await
                .unwrap()
                .unwrap(),
            0
        );
    });
    let client = ManagementClient::connect_with_timeout(&path, Duration::from_millis(100))
        .await
        .unwrap();
    assert!(matches!(client.apps().list().await, Err(Error::Timeout)));
    assert!(matches!(
        client.apps().list().await,
        Err(Error::Disconnected)
    ));
    server.await.unwrap();
}

#[tokio::test]
async fn cancelling_request_discards_partial_connection() {
    let endpoint = Endpoint::new();
    let path = endpoint.path.clone();
    let (observed, wait) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut stream = endpoint.accept().await;
        let _: ClientMessage = wire::read(&mut stream).await.unwrap();
        observed.send(()).unwrap();
        let mut byte = [0];
        assert_eq!(
            timeout(Duration::from_secs(2), stream.read(&mut byte))
                .await
                .unwrap()
                .unwrap(),
            0
        );
    });
    let client = ManagementClient::connect(&path).await.unwrap();
    {
        let request = client.apps().list();
        tokio::pin!(request);
        tokio::select! { _ = wait => {}, result = &mut request => panic!("unexpected response: {result:?}") }
    }
    assert!(matches!(
        client.apps().list().await,
        Err(Error::Disconnected)
    ));
    server.await.unwrap();
}

#[tokio::test]
async fn remote_errors_preserve_connection() {
    let endpoint = Endpoint::new();
    let path = endpoint.path.clone();
    let server = tokio::spawn(async move {
        let mut stream = endpoint.accept().await;
        for id in [1, 2] {
            assert!(
                matches!(wire::read::<ClientMessage>(&mut stream).await.unwrap(), ClientMessage::Request { id: received, .. } if received==id)
            );
            let result = if id == 1 {
                Err(ApiError::new(ErrorCode::NotFound, "unknown app"))
            } else {
                Ok(Reply::Apps(vec![]))
            };
            wire::write(&mut stream, &ServerMessage::Response { id, result })
                .await
                .unwrap();
        }
    });
    let client = ManagementClient::connect(&path).await.unwrap();
    assert!(matches!(client.apps().list().await, Err(Error::Remote(_))));
    assert!(client.apps().list().await.unwrap().is_empty());
    server.await.unwrap();
}

#[tokio::test]
async fn handshake_timeout_and_incompatible_version_are_reported() {
    let endpoint = Endpoint::new();
    assert!(matches!(
        ManagementClient::connect_with_timeout(&endpoint.path, Duration::from_millis(30)).await,
        Err(Error::Timeout)
    ));
    drop(endpoint);
    let endpoint = Endpoint::new();
    let path = endpoint.path.clone();
    let server = tokio::spawn(async move {
        let (mut stream, _) = endpoint.listener.accept().await.unwrap();
        let _: ClientMessage = wire::read(&mut stream).await.unwrap();
        wire::write(&mut stream, &ServerMessage::Hello { version: 999 })
            .await
            .unwrap();
    });
    assert!(matches!(
        ManagementClient::connect(&path).await,
        Err(Error::Protocol(_))
    ));
    server.await.unwrap();
}

#[tokio::test]
async fn invalid_payload_or_truncated_response_closes_connection() {
    for truncate in [false, true] {
        let endpoint = Endpoint::new();
        let path = endpoint.path.clone();
        let server = tokio::spawn(async move {
            let mut stream = endpoint.accept().await;
            let _: ClientMessage = wire::read(&mut stream).await.unwrap();
            if truncate {
                use tokio::io::AsyncWriteExt;
                stream.write_all(&[0, 0, 0, 50, b'{']).await.unwrap();
            } else {
                wire::write(
                    &mut stream,
                    &ServerMessage::Response {
                        id: 1,
                        result: Ok(Reply::Accepted),
                    },
                )
                .await
                .unwrap();
            }
        });
        let client = ManagementClient::connect(&path).await.unwrap();
        assert!(client.apps().list().await.is_err());
        assert!(matches!(
            client.apps().list().await,
            Err(Error::Disconnected)
        ));
        server.await.unwrap();
    }
}
