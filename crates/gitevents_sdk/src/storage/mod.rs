use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;

pub mod volatile;

#[async_trait]
pub trait Storage {
    async fn exists(&self) -> eyre::Result<Option<PathBuf>>;
    async fn allocate(&self) -> eyre::Result<PathBuf>;
}

pub type DynStorage = Arc<dyn Storage + Send + Sync>;
