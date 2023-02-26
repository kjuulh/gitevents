use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::task::JoinSet;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::events::{EventHandler, EventRequest};

#[derive(Clone, Debug)]
pub struct SchedulerOpts {
    pub duration: Duration,
}

impl Default for SchedulerOpts {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(60 * 5),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct CronExecutor {
    opts: SchedulerOpts,
}

impl CronExecutor {
    pub fn new(opts: SchedulerOpts) -> Self {
        Self { opts }
    }

    pub async fn run(
        &self,
        url: &str,
        handlers: &HashMap<uuid::Uuid, Arc<dyn EventHandler + Send + Sync>>,
    ) -> eyre::Result<()> {
        let sched = JobScheduler::new().await?;

        let handlers = handlers.clone();
        let url = url.to_string();

        let job = Job::new_repeated_async(self.opts.duration, move |uuid, _l| {
            let handlers = handlers.clone();
            let url = url.clone();
            Box::pin(async move {
                println!("cron-{uuid}: pulling: {}", &url);

                let mut js: JoinSet<eyre::Result<()>> = JoinSet::new();

                for (uuid, handler) in handlers {
                    let url = url.clone();

                    js.spawn(async move {
                        println!("handler-{uuid}: handling: {}", &url);
                        handler.handle(EventRequest {}).await?;

                        Ok(())
                    });
                }

                while let Some(task) = js.join_next().await {
                    task.unwrap().unwrap();
                }
            })
        })?;

        sched.shutdown_on_ctrl_c();

        sched.add(job).await?;
        sched.start().await?;

        Ok(())
    }
}
