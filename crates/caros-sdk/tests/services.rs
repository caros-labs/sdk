mod support;
use caros_protocol::{AppState, ClientMessage, Reply, Request, ServerMessage, ServiceStatus, wire};
use caros_sdk::ManagementClient;
#[tokio::test]
async fn service_calls_use_typed_contract() {
    let endpoint = support::Endpoint::new();
    let path = endpoint.path.clone();
    let task = tokio::spawn(async move {
        let mut stream = endpoint.accept().await;
        for i in 1..=3 {
            let ClientMessage::Request { id, request } = wire::read(&mut stream).await.unwrap()
            else {
                panic!()
            };
            assert_eq!(id, i);
            match (i, request) {
                (1, Request::AcquireService { name })
                | (2, Request::ServiceStatus { name })
                | (3, Request::ReleaseService { name }) => assert_eq!(name, "receiver"),
                _ => panic!("wrong service request"),
            }
            let reply = if i == 3 {
                Reply::Accepted
            } else {
                Reply::Service(ServiceStatus {
                    name: "receiver".into(),
                    state: AppState::Running,
                    pid: Some(42),
                    owner: None,
                    resources: vec!["audio.output".into()],
                })
            };
            wire::write(
                &mut stream,
                &ServerMessage::Response {
                    id,
                    result: Ok(reply),
                },
            )
            .await
            .unwrap();
        }
    });
    let client = ManagementClient::connect(path).await.unwrap();
    assert_eq!(
        client.services().acquire("receiver").await.unwrap().pid,
        Some(42)
    );
    assert_eq!(
        client
            .services()
            .status("receiver")
            .await
            .unwrap()
            .resources,
        vec!["audio.output"]
    );
    client.services().release("receiver").await.unwrap();
    task.await.unwrap();
}
