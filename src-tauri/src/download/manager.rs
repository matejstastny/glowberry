use std::path::PathBuf;
use std::sync::Arc;

use futures::StreamExt;
use serde::Serialize;
use sha1::Sha1;
use sha2::{Digest, Sha512};
use tokio::sync::Semaphore;

use crate::error::LanternError;

const MAX_CONCURRENT: usize = 10;

#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgress {
    pub file_name: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub files_completed: u32,
    pub files_total: u32,
    pub status: DownloadStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Downloading,
    Verifying,
    Complete,
    Failed,
}

#[derive(Debug, Clone)]
pub enum ExpectedHash {
    Sha512(String),
    Sha1(String),
    None,
}

pub struct DownloadTask {
    pub url: String,
    pub dest: PathBuf,
    pub expected_size: u64,
    pub expected_hash: ExpectedHash,
    pub file_name: String,
}

pub struct DownloadManager {
    client: reqwest::Client,
    semaphore: Arc<Semaphore>,
}

impl DownloadManager {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            client,
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT)),
        }
    }

    /// Download a file, verifying its hash. Overwrites if already exists.
    pub async fn download_file(&self, task: &DownloadTask) -> Result<(), LanternError> {
        let _permit = self.semaphore.acquire().await.unwrap();
        self.download_inner(task).await
    }

    /// Download a file only if it doesn't already exist at the destination.
    pub async fn download_if_missing(&self, task: &DownloadTask) -> Result<(), LanternError> {
        if task.dest.exists() {
            return Ok(());
        }
        let _permit = self.semaphore.acquire().await.unwrap();
        // Re-check after acquiring permit (another task may have downloaded it)
        if task.dest.exists() {
            return Ok(());
        }
        self.download_inner(task).await
    }

    async fn download_inner(&self, task: &DownloadTask) -> Result<(), LanternError> {
        if let Some(parent) = task.dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let part_path = task.dest.with_extension("part");
        let response = self
            .client
            .get(&task.url)
            .send()
            .await?
            .error_for_status()?;

        let mut stream = response.bytes_stream();
        let mut file = tokio::fs::File::create(&part_path).await?;

        use tokio::io::AsyncWriteExt;

        let hash_hex = match &task.expected_hash {
            ExpectedHash::Sha512(_) => {
                let mut hasher = Sha512::new();
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk?;
                    hasher.update(&chunk);
                    file.write_all(&chunk).await?;
                }
                format!("{:x}", hasher.finalize())
            }
            ExpectedHash::Sha1(_) => {
                let mut hasher = Sha1::new();
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk?;
                    hasher.update(&chunk);
                    file.write_all(&chunk).await?;
                }
                format!("{:x}", hasher.finalize())
            }
            ExpectedHash::None => {
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk?;
                    file.write_all(&chunk).await?;
                }
                String::new()
            }
        };

        file.flush().await?;
        drop(file);

        // Verify hash if expected
        match &task.expected_hash {
            ExpectedHash::Sha512(expected) | ExpectedHash::Sha1(expected) => {
                if &hash_hex != expected {
                    let _ = tokio::fs::remove_file(&part_path).await;
                    return Err(LanternError::HashMismatch {
                        file: task.file_name.clone(),
                        expected: expected.clone(),
                        actual: hash_hex,
                    });
                }
            }
            ExpectedHash::None => {}
        }

        tokio::fs::rename(&part_path, &task.dest).await?;
        Ok(())
    }
}
