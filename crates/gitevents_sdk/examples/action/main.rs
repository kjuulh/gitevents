use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use gitevents_sdk::cron::SchedulerOpts;
use gitevents_sdk::events::{EventHandler, EventRequest, EventResponse};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    gitevents_sdk::listen("something")
        .set_scheduler_opts(&SchedulerOpts {
            duration: Duration::from_secs(2),
        })
        .action(|req| async move { Ok(EventResponse {}) })
        .action(other_action)
        .add_handler(Arc::new(TestHandler {}))
        .execute()
        .await?;

    Ok(())
}

async fn other_action(_req: EventRequest) -> eyre::Result<EventResponse> {
    Ok(EventResponse {})
}

pub struct TestHandler;

#[async_trait]
impl EventHandler for TestHandler {
    async fn handle(&self, _req: EventRequest) -> eyre::Result<EventResponse> {
        Ok(EventResponse {})
    }
}
