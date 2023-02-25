use std::sync::Arc;

use async_trait::async_trait;

use crate::events::{ActionFunc, EventHandler, EventRequest, EventResponse};

pub struct ActionEventHandler {
    func: Arc<ActionFunc>,
}

impl ActionEventHandler {
    pub fn new(func: Arc<ActionFunc>) -> Self {
        Self { func }
    }
}

#[async_trait]
impl EventHandler for ActionEventHandler {
    async fn handle(&self, req: EventRequest) -> eyre::Result<EventResponse> {
        (*self.func)(req).await
    }
}
