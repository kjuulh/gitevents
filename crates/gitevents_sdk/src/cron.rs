use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::events::{EventHandler, EventRequest};
use crate::git::{GitEvent, GitProvider};

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
        git_providers: &Vec<Arc<Mutex<dyn GitProvider + Send + Sync>>>,
        handlers: &HashMap<uuid::Uuid, Arc<dyn EventHandler + Send + Sync>>,
    ) -> eyre::Result<()> {
        let sched = JobScheduler::new().await?;

        let git_providers = git_providers.clone();

        let (tx, mut rx) = tokio::sync::broadcast::channel::<GitEvent>(2);

        let mut clone_js: JoinSet<eyre::Result<()>> = JoinSet::new();

        for provider in git_providers.clone() {
            let tx = tx.clone();

            clone_js.spawn(async move {
                tracing::trace!("syncing git_provider");
                if let Some(event) = provider.lock().await.listen().await? {
                    tx.send(event).unwrap();
                }

                Ok(())
            });
        }

        while let Some(task) = clone_js.join_next().await {
            task.unwrap().unwrap();
        }

        let job = Job::new_repeated_async(self.opts.duration, move |uuid, _l| {
            let git_providers = git_providers.clone();
            let tx = tx.clone();
            Box::pin(async move {
                tracing::trace!(uuid = uuid.to_string(), "executing job");

                let mut clone_js: JoinSet<eyre::Result<()>> = JoinSet::new();

                for provider in git_providers {
                    let tx = tx.clone();

                    clone_js.spawn(async move {
                        tracing::trace!("syncing git_provider");
                        if let Some(event) = provider.lock().await.listen().await? {
                            tx.send(event).unwrap();
                        }

                        Ok(())
                    });
                }

                while let Some(task) = clone_js.join_next().await {
                    task.unwrap().unwrap();
                }
            })
        })?;

        let handlers = handlers.clone();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                let handlers = handlers.clone();
                let mut js: JoinSet<eyre::Result<()>> = JoinSet::new();

                for (uuid, handler) in handlers {
                    let event = event.clone();
                    js.spawn(async move {
                        tracing::info!(uuid = uuid.to_string(), "executing task");
                        handler.handle(EventRequest { git: event }).await?;

                        Ok(())
                    });
                }

                while let Some(task) = js.join_next().await {
                    task.unwrap().unwrap();
                }
            }
        });

        sched.shutdown_on_ctrl_c();

        sched.add(job).await?;
        sched.start().await?;

        Ok(())
    }
}
