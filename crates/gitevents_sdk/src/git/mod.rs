pub mod generic;
pub mod simulated;

use std::path::PathBuf;

use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct GitEvent {
    pub commit: String,
    pub path: PathBuf,
}

#[async_trait]
pub trait GitProvider {
    async fn listen(&mut self) -> eyre::Result<Option<GitEvent>>;
}
