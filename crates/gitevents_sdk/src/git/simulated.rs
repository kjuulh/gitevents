use async_trait::async_trait;

use super::{GitEvent, GitProvider};

pub struct GitSimulated {
    repos: Vec<GitEvent>,
}

impl GitSimulated {
    pub fn new() -> Self {
        Self {
            repos: Default::default(),
        }
    }

    pub fn insert(mut self, value: GitEvent) -> Self {
        self
    }
}

#[async_trait]
impl GitProvider for GitSimulated {
    async fn listen(&mut self) -> eyre::Result<Option<GitEvent>> {
        todo!()
    }
}
