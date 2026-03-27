use std::path::PathBuf;
use std::sync::Arc;

use futures::StreamExt;
use serde::Serialize;
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

pub struct DownloadTask {
    pub url: String,
    pub dest: PathBuf,
    pub expected_size: u64,
    pub expected_sha512: Option<String>,
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

    pub async fn download_file(&self, task: &DownloadTask) -> Result<(), LanternError> {
        let _permit = self.semaphore.acquire().await.unwrap();

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
        let mut hasher = Sha512::new();

        use tokio::io::AsyncWriteExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            hasher.update(&chunk);
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        drop(file);

        // Verify hash if expected
        if let Some(expected) = &task.expected_sha512 {
            let actual = format!("{:x}", hasher.finalize());
            if &actual != expected {
                let _ = tokio::fs::remove_file(&part_path).await;
                return Err(LanternError::HashMismatch {
                    file: task.file_name.clone(),
                    expected: expected.clone(),
                    actual,
                });
            }
        }

        tokio::fs::rename(&part_path, &task.dest).await?;
        Ok(())
    }
}
