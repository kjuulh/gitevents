use std::collections::HashMap;
use std::sync::Arc;

use futures::{Future, FutureExt};

use crate::action_event_handler::ActionEventHandler;
use crate::cron::{CronExecutor, SchedulerOpts};
use crate::events::{ActionFunc, EventHandler, EventRequest, EventResponse};

#[allow(dead_code)]
pub struct Builder {
    url: String,
    handlers: HashMap<uuid::Uuid, Arc<dyn EventHandler + Send + Sync>>,
    scheduler_opts: SchedulerOpts,
}

impl Builder {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            handlers: HashMap::new(),
            scheduler_opts: Default::default(),
        }
    }

    pub fn set_scheduler_opts(mut self, opts: &SchedulerOpts) -> Self {
        self.scheduler_opts = opts.clone();
        self
    }

    pub fn action<F, Fut>(mut self, func: F) -> Self
    where
        F: Send + Sync + 'static,
        F: Fn(EventRequest) -> Fut,
        Fut: Send + 'static,
        Fut: Future<Output = eyre::Result<EventResponse>>,
    {
        self.handlers.insert(
            uuid::Uuid::new_v4(),
            Arc::new(ActionEventHandler::new(Arc::new(convert(func)))),
        );
        self
    }

    pub fn add_handler(mut self, handler: Arc<dyn EventHandler + Send + Sync>) -> Self {
        self.handlers.insert(uuid::Uuid::new_v4(), handler);
        self
    }

    pub async fn execute(self) -> eyre::Result<()> {
        CronExecutor::new(self.scheduler_opts)
            .run(&self.url, &self.handlers)
            .await?;

        tokio::signal::ctrl_c().await.and_then(|_| {
            println!();
            println!("received shutdown");
            Ok(())
        })?;

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
