use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use reqwest::Client;
use sha1::{Digest, Sha1};

#[derive(Clone)]
pub struct ThumbnailManager {
    cache_dir: PathBuf,
    client: Client,
    in_flight: Arc<Mutex<HashSet<String>>>,
}

impl ThumbnailManager {
    pub fn new(cache_dir: impl AsRef<Path>) -> Self {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        let _ = std::fs::create_dir_all(&cache_dir);

        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .connect_timeout(Duration::from_secs(10))
            .user_agent("RusticPlayer/0.1.0")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            cache_dir,
            client,
            in_flight: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Computes the unique deterministic hex hash for a URL.
    pub fn hash_url(url: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(url.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Returns the local cached relative or absolute path if the thumbnail exists on disk and is non-empty.
    pub fn get_cached_path(&self, url: &str) -> Option<String> {
        let hash = Self::hash_url(url);
        let path = self.cache_dir.join(format!("{}.jpg", hash));
        if path.exists() {
            if let Ok(meta) = std::fs::metadata(&path) {
                if meta.len() > 0 {
                    return Some(path.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        None
    }

    /// Resolves an optional image URL:
    /// - If None, returns None.
    /// - If already a local file path, returns it.
    /// - If remote (http/https), returns the cached local path if present; otherwise queues download and returns None.
    pub fn resolve_or_queue(&self, url: &Option<String>, on_done: Option<Arc<dyn Fn() + Send + Sync>>) -> Option<String> {
        let url_str = url.as_ref()?;
        if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
            return Some(url_str.clone());
        }

        if let Some(cached) = self.get_cached_path(url_str) {
            return Some(cached);
        }

        // Queue background fetch if not already in flight
        self.prefetch_url(url_str.clone(), on_done);
        None
    }

    /// Prefetches a single URL asynchronously in background.
    pub fn prefetch_url(&self, url: String, on_done: Option<Arc<dyn Fn() + Send + Sync>>) {
        if self.get_cached_path(&url).is_some() {
            if let Some(cb) = on_done {
                cb();
            }
            return;
        }

        {
            let mut in_flight = self.in_flight.lock().unwrap();
            if !in_flight.insert(url.clone()) {
                // Already in-flight
                return;
            }
        }

        let cache_dir = self.cache_dir.clone();
        let client = self.client.clone();
        let in_flight = self.in_flight.clone();

        tokio::spawn(async move {
            let hash = Self::hash_url(&url);
            let final_path = cache_dir.join(format!("{}.jpg", hash));
            let tmp_path = cache_dir.join(format!("{}.tmp", hash));

            let res = client.get(&url).send().await;
            match res {
                Ok(response) if response.status().is_success() => {
                    if let Ok(bytes) = response.bytes().await {
                        if !bytes.is_empty() {
                            let _ = std::fs::create_dir_all(&cache_dir);
                            let mut wrote_cropped = false;
                            // If non-square (e.g. 16:9 YouTube video thumbnail), center-crop to 1:1 square
                            if let Ok(dyn_img) = image::load_from_memory(&bytes) {
                                let w = dyn_img.width();
                                let h = dyn_img.height();
                                if w > 0 && h > 0 && w != h {
                                    let min_dim = w.min(h);
                                    let x = (w - min_dim) / 2;
                                    let y = (h - min_dim) / 2;
                                    let cropped = dyn_img.crop_imm(x, y, min_dim, min_dim);
                                    let mut buf = std::io::Cursor::new(Vec::new());
                                    if cropped.write_to(&mut buf, image::ImageFormat::Jpeg).is_ok() {
                                        if std::fs::write(&tmp_path, buf.into_inner()).is_ok() {
                                            let _ = std::fs::rename(&tmp_path, &final_path);
                                            wrote_cropped = true;
                                        }
                                    }
                                }
                            }
                            if !wrote_cropped {
                                if std::fs::write(&tmp_path, &bytes).is_ok() {
                                    let _ = std::fs::rename(&tmp_path, &final_path);
                                }
                            }
                        }
                    }
                }
                Ok(response) => {
                    tracing::warn!(target: "app::thumbnails", "Failed to fetch thumbnail {} status {}", url, response.status());
                }
                Err(e) => {
                    tracing::warn!(target: "app::thumbnails", "Error downloading thumbnail {}: {:?}", url, e);
                }
            }

            {
                let mut guard = in_flight.lock().unwrap();
                guard.remove(&url);
            }

            if let Some(cb) = on_done {
                cb();
            }
        });
    }

    /// Prefetches a batch of URLs, notifying when complete.
    pub fn prefetch_batch(&self, urls: Vec<String>, on_batch_done: Option<Arc<dyn Fn() + Send + Sync>>) {
        let uncached: Vec<String> = urls
            .into_iter()
            .filter(|u| u.starts_with("http://") || u.starts_with("https://"))
            .filter(|u| self.get_cached_path(u).is_none())
            .collect();

        if uncached.is_empty() {
            if let Some(cb) = on_batch_done {
                cb();
            }
            return;
        }

        let completed = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        for url in uncached {
            let completed_clone = completed.clone();
            let on_batch_done_clone = on_batch_done.clone();
            let cb = Arc::new(move || {
                let _prev = completed_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if let Some(ref done) = on_batch_done_clone {
                    done();
                }
            });
            self.prefetch_url(url, Some(cb));
        }
    }

    /// Prefetches a batch of URLs, returning an unbounded mpsc receiver that emits `()` on each downloaded thumbnail.
    pub fn prefetch_batch_channel(&self, urls: Vec<String>) -> tokio::sync::mpsc::UnboundedReceiver<()> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let uncached: Vec<String> = urls
            .into_iter()
            .filter(|u| u.starts_with("http://") || u.starts_with("https://"))
            .filter(|u| self.get_cached_path(u).is_none())
            .collect();

        if uncached.is_empty() {
            return rx;
        }

        for url in uncached {
            let tx_clone = tx.clone();
            let cb = Arc::new(move || {
                let _ = tx_clone.send(());
            });
            self.prefetch_url(url, Some(cb));
        }

        rx
    }

    /// Asynchronously fetches a thumbnail URL and returns the local cached path when ready.
    pub async fn fetch_and_get_path(&self, url: String) -> Option<String> {
        if let Some(p) = self.get_cached_path(&url) {
            return Some(p);
        }

        let (tx, rx) = tokio::sync::oneshot::channel();
        let tx_cell = Arc::new(Mutex::new(Some(tx)));
        self.prefetch_url(url.clone(), Some(Arc::new(move || {
            if let Ok(mut guard) = tx_cell.lock() {
                if let Some(sender) = guard.take() {
                    let _ = sender.send(());
                }
            }
        })));

        let _ = rx.await;
        self.get_cached_path(&url)
    }
}
