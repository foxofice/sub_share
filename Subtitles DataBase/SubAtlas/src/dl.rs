// Database download powered by ufetch-core (adaptive multi-connection engine:
// dynamic segmentation, slow-segment takeover, mirror/retry, resume).
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub const DB_URL_SHORT: &str = "github.com/foxofice/sub_share";

/// Preferred compressed database (JSON + zstd), same folder as db.xml.
pub const DB_JSON_URL: &str =
    "https://raw.githubusercontent.com/foxofice/sub_share/master/Subtitles%20DataBase/Files/db.json.zst";

pub const DB_URL: &str =
    "https://github.com/foxofice/sub_share/raw/master/Subtitles%20DataBase/Files/db.xml";

#[derive(Debug, Clone, PartialEq)]
pub enum DownloadStatus {
    Idle,
    Progress { received: u64, total: u64 },
    Done,
    Failed(String),
    Cancelled,
}

pub struct Downloader {
    pub status: Arc<Mutex<DownloadStatus>>,
    pub stop: Arc<AtomicBool>,
    pub bytes: Arc<AtomicU64>,
    pub total: Arc<AtomicU64>,
    pub speed: Arc<AtomicU64>,
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(DownloadStatus::Idle)),
            stop: Arc::new(AtomicBool::new(false)),
            bytes: Arc::new(AtomicU64::new(0)),
            total: Arc::new(AtomicU64::new(0)),
            speed: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn cancel(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    pub fn status(&self) -> DownloadStatus {
        self.status.lock().unwrap().clone()
    }

    /// Blocking download; run on a background thread. `dest_path` decides the
    /// final file name (e.g. Files\db.json.zst).
    pub fn download(
        &self,
        url: &str,
        dest_path: &Path,
        connections: Option<u32>,
    ) -> Result<(), String> {
        *self.status.lock().unwrap() = DownloadStatus::Idle;
        self.stop.store(false, Ordering::Relaxed);
        self.bytes.store(0, Ordering::Relaxed);
        self.total.store(0, Ordering::Relaxed);
        self.speed.store(0, Ordering::Relaxed);

        let dest_dir = dest_path
            .parent()
            .map(|p| p.to_path_buf())
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new(".").to_path_buf());
        let file_name = dest_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .ok_or_else(|| "invalid destination path".to_string())?;
        let _ = std::fs::create_dir_all(&dest_dir);

        let status = self.status.clone();
        let bytes = self.bytes.clone();
        let total = self.total.clone();
        let speed = self.speed.clone();
        let stop = self.stop.clone();
        let url = url.to_string();

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("tokio runtime: {e}"))?;

        let result = rt.block_on(async move {
            let manager = ufetch_core::DownloadManager::new();
            let mut request = ufetch_core::DownloadRequest::new(url, dest_dir)
                .file_name(file_name)
                .overwrite(true)
                .keep_partial(false);
            if let Some(n) = connections.filter(|n| *n > 0) {
                request = request.connections(n);
            }
            let task = manager.download(request);
            let mut rx = task.subscribe();
            let control = task.control().clone();

            let done = Arc::new(AtomicBool::new(false));
            let done_cancel = done.clone();
            let stop_cancel = stop.clone();
            let cancel_task = tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    if done_cancel.load(Ordering::Relaxed) {
                        break;
                    }
                    if stop_cancel.load(Ordering::Relaxed) {
                        control.cancel();
                        break;
                    }
                }
            });

            let progress_task = tokio::spawn(async move {
                loop {
                    if rx.changed().await.is_err() {
                        break;
                    }
                    let p = rx.borrow_and_update().clone();
                    bytes.store(p.downloaded, Ordering::Relaxed);
                    total.store(p.total, Ordering::Relaxed);
                    speed.store(p.speed_bps, Ordering::Relaxed);
                    *status.lock().unwrap() = DownloadStatus::Progress {
                        received: p.downloaded,
                        total: p.total,
                    };
                    if p.state.is_terminal() {
                        break;
                    }
                }
            });

            let res = task.wait().await;
            done.store(true, Ordering::Relaxed);
            let _ = cancel_task.await;
            let _ = progress_task.await;
            res
        });

        match result {
            Ok(_outcome) => {
                *self.status.lock().unwrap() = DownloadStatus::Done;
                Ok(())
            }
            Err(e) => {
                if self.stop.load(Ordering::Relaxed) {
                    *self.status.lock().unwrap() = DownloadStatus::Cancelled;
                    Err("cancelled".to_string())
                } else {
                    let msg = e.to_string();
                    *self.status.lock().unwrap() = DownloadStatus::Failed(msg.clone());
                    Err(msg)
                }
            }
        }
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Live test through the embedded ufetch engine (skipped when offline).
    #[test]
    fn download_real_db_json_zst() {
        let dir = std::env::temp_dir().join("subdb_dl_test");
        std::fs::create_dir_all(&dir).unwrap();
        let dest = dir.join("db.json.zst");
        let _ = std::fs::remove_file(&dest);
        let dl = Downloader::new();
        match dl.download(DB_JSON_URL, &dest, None) {
            Ok(()) => {
                let meta = std::fs::metadata(&dest).unwrap();
                println!("downloaded {} bytes via ufetch", meta.len());
                assert!(meta.len() > 100_000);
                let head = std::fs::read(&dest).unwrap();
                // zstd magic number
                assert_eq!(&head[..4], &[0x28, 0xB5, 0x2F, 0xFD]);
                // and it must parse as our database
                let subs = crate::db::read_db_json_zst(&dest).expect("parse downloaded db");
                assert!(subs.len() > 4000);
            }
            Err(e) => {
                let offline = e.contains("connect")
                    || e.contains("dns")
                    || e.contains("timed out")
                    || e.contains("network")
                    || e.contains("timeout");
                assert!(offline, "download failed unexpectedly: {e}");
                eprintln!("offline, skipped: {e}");
            }
        }
        std::fs::remove_file(&dest).ok();
    }
}
