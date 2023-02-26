use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::task::JoinSet;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::events::{EventHandler, EventRequest};
use crate::git::GitProvider;

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
        git_providers: &Vec<Arc<dyn GitProvider + Send + Sync>>,
        handlers: &HashMap<uuid::Uuid, Arc<dyn EventHandler + Send + Sync>>,
    ) -> eyre::Result<()> {
        let sched = JobScheduler::new().await?;

        let handlers = handlers.clone();
        let git_providers = git_providers.clone();

        let job = Job::new_repeated_async(self.opts.duration, move |uuid, _l| {
            let handlers = handlers.clone();
            let git_providers = git_providers.clone();
            Box::pin(async move {
                tracing::trace!(uuid = uuid.to_string(), "executing job");
                let mut js: JoinSet<eyre::Result<()>> = JoinSet::new();

                for (uuid, handler) in handlers {
                    js.spawn(async move {
                        tracing::trace!(uuid = uuid.to_string(), "executing task");
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
