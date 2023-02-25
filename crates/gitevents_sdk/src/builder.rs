use std::sync::Arc;

use futures::{Future, FutureExt};

use crate::action_event_handler::ActionEventHandler;
use crate::events::{ActionFunc, EventHandler, EventRequest, EventResponse};

#[allow(dead_code)]
pub struct Builder {
    url: String,
    handlers: Vec<Arc<dyn EventHandler + Send + Sync>>,
}

impl Builder {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            handlers: Vec::new(),
        }
    }

    pub fn action<F, Fut>(mut self, func: F) -> Self
    where
        F: Send + Sync + 'static,
        F: Fn(EventRequest) -> Fut,
        Fut: Send + 'static,
        Fut: Future<Output = eyre::Result<EventResponse>>,
    {
        self.handlers
            .push(Arc::new(ActionEventHandler::new(Arc::new(convert(func)))));
        self
    }

    pub async fn execute(self) -> eyre::Result<()> {
        Ok(())
    }
}

fn convert<F, Fut>(func: F) -> ActionFunc
where
    F: Send + Sync + 'static,
    F: Fn(EventRequest) -> Fut,
    Fut: Send + 'static,
    Fut: Future<Output = eyre::Result<EventResponse>>,
{
    Box::new(move |context| func(context).boxed())
}
