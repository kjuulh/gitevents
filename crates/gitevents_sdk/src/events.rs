use async_trait::async_trait;
use futures::future::BoxFuture;

#[derive(Debug, Clone)]
pub struct EventRequest {}

#[derive(Debug, Clone)]
pub struct EventResponse {}

#[async_trait] // Will use this trait for now. async trait fns will probably be available soon'ish.
pub trait EventHandler {
    async fn handle(&self, req: EventRequest) -> eyre::Result<EventResponse>;
}

pub type ActionFunc =
    Box<dyn Send + Sync + Fn(EventRequest) -> BoxFuture<'static, eyre::Result<EventResponse>>>;
