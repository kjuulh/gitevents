use std::path::PathBuf;
use std::sync::Arc;

pub mod volatile;

pub trait Storage {
    fn allocate(&mut self) -> eyre::Result<PathBuf>;
}

pub type DynStorage = Arc<dyn Storage + Send + Sync>;
