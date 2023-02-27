use async_trait::async_trait;
use tokio::sync::Mutex;

use super::{GitEvent, GitProvider};

pub struct GitSimulated {
    mutex: Mutex<()>,
    events: Vec<GitEvent>,
}

impl GitSimulated {
    pub fn new() -> Self {
        Self {
            mutex: Mutex::new(()),
            events: Default::default(),
        }
    }

    pub fn insert(mut self, value: GitEvent) -> Self {
        self.events.push(value);
        self
    }
}

#[async_trait]
impl GitProvider for GitSimulated {
    async fn listen(&mut self) -> eyre::Result<Option<GitEvent>> {
        let mutex = self.mutex.lock().await;
        let event = self.events.pop();
        drop(mutex);
        return Ok(event);
    }
}
