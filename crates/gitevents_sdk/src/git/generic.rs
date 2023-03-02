use std::process::Stdio;
use std::sync::Arc;

use async_trait::async_trait;
use git2::Repository;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

use crate::storage::volatile::VolatileStorage;
use crate::storage::DynStorage;

use super::{GitEvent, GitProvider};

pub struct GitGeneric {
    url: String,
    storage: DynStorage,
    progress: Mutex<Option<String>>,
}

impl GitGeneric {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            storage: Arc::new(VolatileStorage::new()),
            progress: Mutex::new(None),
        }
    }
}

#[async_trait]
impl GitProvider for GitGeneric {
    async fn listen(&mut self) -> eyre::Result<Option<GitEvent>> {
        match self.storage.exists().await? {
            Some(path) => {
                let mut cmd = tokio::process::Command::new("git")
                    .args(&["pull"])
                    .current_dir(&path)
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
                        tracing::debug!(line = line, "out: git pull");
                    }
                });

                tokio::spawn(async move {
                    while let Ok(Some(line)) = errreader.next_line().await {
                        tracing::debug!(line = line, "err: git pull");
                    }
                });

                let status = cmd.wait().await?.to_string();
                tracing::debug!(status = status, "git pull finished");

                let mut p = self.progress.lock().await;
                match p.as_mut() {
                    Some(p) => {
                        let repo = Repository::open(path)?;
                        let head = repo.head()?.target().unwrap();
                        let mut revwalk = repo.revwalk()?;
                        revwalk.set_sorting(git2::Sort::NONE)?; //| git2::Sort::REVERSE)?;
                        let start = git2::Oid::from_str(p)?;
                        revwalk.hide(start)?;
                        revwalk.push(head)?;

                        if let Some(rev) = revwalk.next() {
                            let revstr = rev?.to_string();
                            tracing::trace!(progress = &revstr, "storing progress");
                            dbg!(&revstr);
                            *p = revstr.clone();

                            return Ok(Some(GitEvent { commit: revstr }));
                        }
                    }
                    None => {
                        eyre::bail!(
                            "inconsistency found, object should not already have progress stored"
                        );
                    }
                }

                Ok(None)
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

                let mut p = self.progress.lock().await;
                match p.as_mut() {
                    Some(_) => eyre::bail!(
                        "inconsistency found, object should not already have progress stored"
                    ),
                    None => {
                        let repo = Repository::open(path)?;
                        let head = repo.head()?;
                        let revstr = head.target().unwrap().to_string();
                        tracing::trace!(progress = &revstr, "storing progress");
                        *p = Some(revstr);
                    }
                }

                drop(p);

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

        let mut file_path2 = tempdir.clone();
        file_path2.push("readme2.md");
        write(file_path2, "Some file").unwrap();

        git_commit_all(&tempdir, "next commit").await.unwrap();

        let mut git = GitGeneric::new(tempdir.to_str().unwrap());
        let event = git.listen().await.unwrap();

        assert!(event.is_none());
        assert!(logs_contain("git clone finished"));
        assert!(logs_contain("err: git clone"));

        let mut file_path3 = tempdir.clone();
        file_path3.push("readme3.md");
        write(file_path3, "Some file").unwrap();

        git_commit_all(&tempdir, "next commit 3").await.unwrap();

        let event = git.listen().await.unwrap();

        assert!(event.is_some());
        assert!(logs_contain("git pull finished"));
        assert!(logs_contain("err: git pull"));
        assert!(logs_contain("storing progress"));

        let mut file_path3 = tempdir.clone();
        file_path3.push("readme3.md");
        write(file_path3, "Some file123").unwrap();

        git_commit_all(&tempdir, "next commit 4").await.unwrap();

        let event = git.listen().await.unwrap();

        assert!(event.is_some());
        assert!(logs_contain("git pull finished"));
        assert!(logs_contain("err: git pull"));
        assert!(logs_contain("storing progress"));

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
