use caros_sdk::{AppId, ManagementClient};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        return Err("usage: manage SOCKET list|watch|status|start|stop [APP_UUID]".into());
    }
    let client = ManagementClient::connect(&args[0]).await?;
    match args[1].as_str() {
        "list" => {
            for app in client.apps().list().await? {
                println!("{} {} {:?}", app.id, app.name, app.state);
            }
        }
        "watch" => {
            let mut events = client.subscribe().await?;
            println!("snapshot at {}: {:?}", events.sequence(), events.snapshot());
            loop {
                println!("{:?}", events.next().await?);
            }
        }
        action => {
            let id: AppId = args.get(2).ok_or("missing app UUID")?.parse()?;
            match action {
                "status" => println!("{:?}", client.apps().status(id).await?),
                "start" => client.apps().start(id).await?,
                "stop" => client.apps().stop(id).await?,
                _ => return Err("unknown action".into()),
            }
        }
    }
    Ok(())
}
