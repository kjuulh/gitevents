use async_trait::async_trait;

use super::{GitEvent, GitProvider};

pub struct GitSimulated {
    events: Vec<GitEvent>,
}

impl GitSimulated {
    pub fn new() -> Self {
        Self {
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
        Ok(self.events.pop())
    }
}
