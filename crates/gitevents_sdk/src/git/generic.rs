use std::sync::Arc;

use async_trait::async_trait;

use crate::storage::volatile::VolatileStorage;
use crate::storage::DynStorage;

use super::{GitEvent, GitProvider};

pub struct GitGeneric {
    url: String,
    storage: DynStorage,
}

impl GitGeneric {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            storage: Arc::new(VolatileStorage::new()),
        }
    }
}

#[async_trait]
impl GitProvider for GitGeneric {
    async fn listen(&mut self) -> eyre::Result<Option<GitEvent>> {
        todo!()
    }
}
