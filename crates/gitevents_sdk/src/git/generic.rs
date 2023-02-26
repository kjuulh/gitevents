use std::process::Stdio;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, BufReader};

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
        match self.storage.exists().await? {
            Some(_path) => {
                todo!("update repo");
            }
            None => {
                let path = self.storage.allocate().await?;

                let mut cmd = tokio::process::Command::new("git")
                    .args(&[
                        "clone",
                        self.url.as_str(),
                        path.to_str()
                            .ok_or(eyre::anyhow!("could not transform path into str"))?,
                    ])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?;

                let stdout = cmd
                    .stdout
                    .take()
                    .ok_or(eyre::anyhow!("failed to capture stdout of cmd"))?;
                let stderr = cmd
                    .stderr
                    .take()
                    .ok_or(eyre::anyhow!("failed to capture stdout of cmd"))?;
                let mut reader = BufReader::new(stdout).lines();
                let mut errreader = BufReader::new(stderr).lines();

                tokio::spawn(async move {
                    while let Ok(Some(line)) = reader.next_line().await {
                        tracing::debug!(line = line, "out: git clone");
                    }
                });

                tokio::spawn(async move {
                    while let Ok(Some(line)) = errreader.next_line().await {
                        tracing::debug!(line = line, "err: git clone");
                    }
                });

                let status = cmd.wait().await?.to_string();
                tracing::debug!(status = status, "git clone finished");

                //TODO: Store progress

                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;
    use std::fs::write;
    use std::path::PathBuf;

    use tokio::fs::{create_dir_all, remove_dir_all};
    use tracing::info;
    use tracing_test::traced_test;

    use crate::git::GitProvider;

    use super::GitGeneric;

    #[tokio::test]
    #[traced_test]
    async fn test_can_listen_to_repo() {
        let tempdir = git_init().await.unwrap();

        let mut file_path = tempdir.clone();
        file_path.push("readme.md");
        write(file_path, "Some file").unwrap();

        git_commit_all(&tempdir, "initial file").await.unwrap();

        let mut git = GitGeneric::new(tempdir.to_str().unwrap());
        let event = git.listen().await.unwrap();

        assert!(event.is_none());
        assert!(logs_contain("git clone finished"));
        assert!(logs_contain("err: git clone"));

        remove_dir_all(tempdir).await.unwrap();
    }

    async fn git_init() -> eyre::Result<PathBuf> {
        let mut tempdir = temp_dir();
        tempdir.push(uuid::Uuid::new_v4().to_string());

        create_dir_all(&tempdir).await.unwrap();
        let output = tokio::process::Command::new("git")
            .args(&["init", tempdir.to_str().unwrap()])
            .output()
            .await?;
        println!("{}", std::str::from_utf8(output.stdout.as_slice()).unwrap());

        Ok(tempdir)
    }

    async fn git_commit_all(dir: &PathBuf, message: impl Into<String>) -> eyre::Result<()> {
        let output = tokio::process::Command::new("git")
            .args(&["add", "."])
            .current_dir(dir)
            .output()
            .await
            .unwrap();
        info!("{}", std::str::from_utf8(output.stdout.as_slice()).unwrap());

        let output = tokio::process::Command::new("git")
            .args(&["commit", "-m", &message.into()])
            .current_dir(dir)
            .output()
            .await
            .unwrap();
        info!("{}", std::str::from_utf8(output.stdout.as_slice()).unwrap());

        Ok(())
    }
}
