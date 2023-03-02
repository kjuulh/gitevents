use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use gitevents_sdk::{
    cron::SchedulerOpts,
    events::{EventHandler, EventRequest, EventResponse},
};
use tracing::Level;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(Level::TRACE)
        .init();

    gitevents_sdk::builder::Builder::new()
        .set_generic_git_url("git@git.front.kjuulh.io:kjuulh/gitevents.git")
        .set_scheduler_opts(&SchedulerOpts {
            // Duration must not be lower than 1 second, otherwise async runtime won't proceed
            duration: Duration::from_secs(10),
        })
        .action(|_req| async move { Ok(EventResponse {}) })
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
