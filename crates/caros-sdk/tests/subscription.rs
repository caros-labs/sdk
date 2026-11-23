mod support;
use caros_protocol::{
    AppEvent, AppId, ClientMessage, EventKind, LifecycleEvent, Reply, ServerMessage, wire,
};
use caros_sdk::{Error, ManagementClient};
use std::time::Duration;
use support::Endpoint;
use tokio::{net::UnixStream, time::timeout};

async fn subscribed(endpoint: &Endpoint) -> (UnixStream, UnixStream) {
    let management = endpoint.accept().await;
    let mut events = endpoint.accept().await;
    let _: ClientMessage = wire::read(&mut events).await.unwrap();
    wire::write(
        &mut events,
        &ServerMessage::Response {
            id: 1,
            result: Ok(Reply::Subscribed {
                apps: vec![],
                sequence: 10,
            }),
        },
    )
    .await
    .unwrap();
    (management, events)
}

#[tokio::test]
async fn explicit_lag_and_sequence_gaps_require_resubscription() {
    for message in [
        ServerMessage::Lagged { missed: 4 },
        ServerMessage::Event(LifecycleEvent {
            sequence: 12,
            event: AppEvent {
                app_id: AppId::new(),
                kind: EventKind::Stopped,
            },
        }),
    ] {
        let endpoint = Endpoint::new();
        let path = endpoint.path.clone();
        let server = tokio::spawn(async move {
            let (_management, mut events) = subscribed(&endpoint).await;
            wire::write(&mut events, &message).await.unwrap();
        });
        let client = ManagementClient::connect(&path).await.unwrap();
        let mut events = client.subscribe().await.unwrap();
        assert_eq!(events.sequence(), 10);
        assert!(matches!(
            events.next().await,
            Err(Error::Lagged { .. }) | Err(Error::Protocol(_))
        ));
        assert!(matches!(events.next().await, Err(Error::Disconnected)));
        server.await.unwrap();
    }
}

#[tokio::test]
async fn cancelling_event_read_closes_subscription() {
    let endpoint = Endpoint::new();
    let path = endpoint.path.clone();
    let server = tokio::spawn(async move {
        let (_management, mut events) = subscribed(&endpoint).await;
        use tokio::io::AsyncReadExt;
        let mut byte = [0];
        assert_eq!(
            timeout(Duration::from_secs(2), events.read(&mut byte))
                .await
                .unwrap()
                .unwrap(),
            0
        );
    });
    let client = ManagementClient::connect(&path).await.unwrap();
    let mut events = client.subscribe().await.unwrap();
    assert!(
        timeout(Duration::from_millis(30), events.next())
            .await
            .is_err()
    );
    assert!(matches!(events.next().await, Err(Error::Disconnected)));
    server.await.unwrap();
}
