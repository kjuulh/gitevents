pub mod generic;
pub mod simulated;

use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct GitEvent {}

#[async_trait]
pub trait GitProvider {
    async fn listen(&mut self) -> eyre::Result<Option<GitEvent>>;
}
