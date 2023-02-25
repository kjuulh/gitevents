use std::sync::Arc;

use async_trait::async_trait;
use gitevents_sdk::events::{EventHandler, EventRequest, EventResponse};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    gitevents_sdk::listen("something")
        .action(|req| async move {
            println!("{:?}", req);
            todo!()
        })
        .action(other_action)
        .add_handler(Arc::new(TestHandler {}))
        .execute()
        .await?;

    Ok(())
}

async fn other_action(_req: EventRequest) -> eyre::Result<EventResponse> {
    todo!()
}

pub struct TestHandler;

#[async_trait]
impl EventHandler for TestHandler {
    async fn handle(&self, _req: EventRequest) -> eyre::Result<EventResponse> {
        todo!()
    }
}
