use async_trait::async_trait;

use super::{GitEvent, GitProvider};

pub struct GitGeneric {
    url: String,
}

impl GitGeneric {
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

#[async_trait]
impl GitProvider for GitGeneric {
    async fn listen(&mut self) -> eyre::Result<Option<GitEvent>> {
        todo!()
    }
}
