use std::env::temp_dir;
use std::path::PathBuf;

use super::Storage;

pub struct VolatileStorage {
    pub dir: PathBuf,
    pub paths: Vec<PathBuf>,
}

impl VolatileStorage {
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
}

impl Drop for VolatileStorage {
    fn drop(&mut self) {
        tracing::trace!("cleaning up directory: {}", self.dir.display());
        std::fs::remove_dir_all(&self.dir).unwrap();
    }
}

impl Storage for VolatileStorage {
    fn allocate(&mut self) -> eyre::Result<PathBuf> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use tracing_test::traced_test;

    use super::VolatileStorage;

    #[test]
    #[traced_test]
    fn test_create_volatile_storage() {
        let _ = VolatileStorage::new();

        assert!(logs_contain("creating volatile storage: "));
        assert!(logs_contain("cleaning up directory: "));
    }
}
