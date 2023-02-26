use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use gitevents_sdk::{
    cron::SchedulerOpts,
    events::{EventHandler, EventRequest, EventResponse},
    git::{simulated::GitSimulated, GitEvent},
};
use tokio::sync::Mutex;
use tracing::Level;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(Level::TRACE)
        .init();

    let simulated_git = GitSimulated::new()
        .insert(GitEvent {})
        .insert(GitEvent {})
        .insert(GitEvent {});

    gitevents_sdk::builder::Builder::new()
        .add_git_provider(Arc::new(Mutex::new(simulated_git)))
        .set_scheduler_opts(&SchedulerOpts {
            duration: Duration::from_secs(2),
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
