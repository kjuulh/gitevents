use gitevents_sdk::events::{EventRequest, EventResponse};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    gitevents_sdk::listen("something")
        .action(|req| async move {
            println!("{:?}", req);
            todo!()
        })
        .action(other_action)
        .execute()
        .await?;

    Ok(())
}

async fn other_action(_req: EventRequest) -> eyre::Result<EventResponse> {
    todo!()
}
