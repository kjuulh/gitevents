use std::env::temp_dir;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use super::Storage;

pub struct InnerVolatileStorage {
    pub dir: PathBuf,
    pub paths: Vec<PathBuf>,
}

impl InnerVolatileStorage {
    pub fn new() -> Self {
        let mut dir = temp_dir();
        let id = uuid::Uuid::new_v4();
        dir.push("gitevents/storage");
        dir.push(id.to_string());

        tracing::trace!("creating volatile storage: {}", dir.display());
        std::fs::create_dir_all(&dir).unwrap();

        Self {
            dir,
            paths: Default::default(),
        }
    }

    pub async fn allocate(&mut self) -> eyre::Result<PathBuf> {
        let mut new_dir = self.dir.clone();
        let new_dir_id = uuid::Uuid::new_v4();
        new_dir.push(new_dir_id.to_string());

        std::fs::create_dir_all(&new_dir)?;
        self.paths.push(new_dir.clone());

        tracing::trace!(new_dir = new_dir.display().to_string(), "allocating dir");

        Ok(new_dir)
    }

    pub async fn exists(&self) -> eyre::Result<Option<PathBuf>> {
        match self.paths.first() {
            Some(path) => {
                if path.exists() {
                    Ok(Some(path.clone()))
                } else {
                    eyre::bail!("path doesn't exist: {}", path.display())
                }
            }
            None => Ok(None),
        }
    }
}

impl Drop for InnerVolatileStorage {
    fn drop(&mut self) {
        tracing::trace!("cleaning up directory: {}", self.dir.display());
        std::fs::remove_dir_all(&self.dir).unwrap();
    }
}

pub struct VolatileStorage {
    pub inner: Arc<Mutex<InnerVolatileStorage>>,
}

impl VolatileStorage {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(InnerVolatileStorage::new())),
        }
    }
}

#[async_trait]
impl Storage for VolatileStorage {
    async fn allocate(&self) -> eyre::Result<PathBuf> {
        self.inner.lock().await.allocate().await
    }

    async fn exists(&self) -> eyre::Result<Option<PathBuf>> {
        self.inner.lock().await.exists().await
    }
}

#[cfg(test)]
mod test {
    use tracing_test::traced_test;

    use crate::storage::Storage;

    use super::VolatileStorage;

    #[test]
    #[traced_test]
    fn test_create_volatile_storage() {
        let _ = VolatileStorage::new();

        assert!(logs_contain("creating volatile storage: "));
        assert!(logs_contain("cleaning up directory: "));
    }

    #[tokio::test]
    #[traced_test]
    async fn test_volatile_storage_is_created_and_cleaned_up() {
        let storage = VolatileStorage::new();
        storage.allocate().await.unwrap();

        let inner = storage.inner.lock().await;

        assert!(inner.dir.exists());
        assert!(inner.paths.first().unwrap().exists());

        assert!(logs_contain("allocating dir"));
    }
}
