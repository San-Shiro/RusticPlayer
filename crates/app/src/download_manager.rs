// crates/app/src/download_manager.rs
//! Manages offline audio downloads and metadata indexing for RusticPlayer.
//!
//! Features:
//! - Crash-consistent atomic streaming (.tmp -> final audio file).
//! - Atomic metadata indexing (`metadata.json.tmp` -> `metadata.json`).
//! - Fast O(1) in-memory index queries on playback hot path.
//! - Startup reconciliation against disk.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use rustic_core::api::Track;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadEntry {
    pub track: Track,
    pub relative_audio_path: String,
    pub relative_artwork_path: Option<String>,
    pub file_size_bytes: u64,
    pub mime_type: String,
    pub downloaded_at: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DownloadIndex {
    pub version: u32,
    pub entries: HashMap<String, DownloadEntry>,
}

pub struct DownloadManager {
    base_dir: PathBuf,
    index: DownloadIndex,
}

impl DownloadManager {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        let base_dir = base_dir.into();
        let audio_dir = base_dir.join("audio");
        let artwork_dir = base_dir.join("artwork");

        let _ = fs::create_dir_all(&audio_dir);
        let _ = fs::create_dir_all(&artwork_dir);

        let metadata_path = base_dir.join("metadata.json");
        let index = if metadata_path.exists() {
            match fs::read(&metadata_path) {
                Ok(bytes) => serde_json::from_slice::<DownloadIndex>(&bytes).unwrap_or_default(),
                Err(e) => {
                    tracing::warn!(target: "app::download_manager", "Failed to read metadata.json: {e}");
                    DownloadIndex::default()
                }
            }
        } else {
            DownloadIndex::default()
        };

        let mut mgr = Self { base_dir, index };
        mgr.reconcile_startup();
        mgr
    }

    /// Reconciles the in-memory index with actual files on disk.
    /// Removes stale index entries whose files have been deleted,
    /// and cleans up old `.tmp` files.
    fn reconcile_startup(&mut self) {
        let base = &self.base_dir;
        let mut stale_keys = Vec::new();

        for (id, entry) in &self.index.entries {
            let full_audio = base.join(&entry.relative_audio_path);
            if !full_audio.exists() {
                stale_keys.push(id.clone());
            }
        }

        if !stale_keys.is_empty() {
            for k in stale_keys {
                self.index.entries.remove(&k);
            }
            let _ = self.save_index_atomic();
        }

        // Clean any .tmp files in audio dir
        let audio_dir = base.join("audio");
        if let Ok(entries) = fs::read_dir(audio_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "tmp" {
                        let _ = fs::remove_file(path);
                    }
                }
            }
        }
    }

    /// O(1) in-memory check for whether a track is downloaded.
    /// Returns the absolute or canonical PathBuf to the audio file if present.
    pub fn is_downloaded(&self, video_id: &str) -> Option<PathBuf> {
        let entry = self.index.entries.get(video_id)?;
        let full_path = self.base_dir.join(&entry.relative_audio_path);
        if full_path.exists() {
            Some(full_path)
        } else {
            None
        }
    }

    /// Returns a list of all downloaded tracks in the index.
    pub fn get_downloaded_tracks(&self) -> Vec<Track> {
        self.index.entries.values().map(|e| e.track.clone()).collect()
    }

    /// Total storage used by downloads in bytes.
    pub fn get_total_bytes(&self) -> u64 {
        self.index.entries.values().map(|e| e.file_size_bytes).sum()
    }

    /// Total number of tracks downloaded.
    pub fn get_total_tracks(&self) -> usize {
        self.index.entries.len()
    }

    /// Streams audio from `stream_url` to a `.tmp` file and atomically renames on completion.
    /// Also saves artwork if available, and updates `metadata.json`.
    pub async fn download_track(
        &mut self,
        track: Track,
        stream_url: &str,
        client: &reqwest::Client,
    ) -> Result<PathBuf, String> {
        let video_id = track.id.clone();
        let audio_dir = self.base_dir.join("audio");
        let artwork_dir = self.base_dir.join("artwork");

        let _ = fs::create_dir_all(&audio_dir);
        let _ = fs::create_dir_all(&artwork_dir);

        // Determine extension from URL or fallback to m4a
        let ext = if stream_url.contains("mime=audio%2Fwebm") || stream_url.contains("audio/webm") {
            "webm"
        } else {
            "m4a"
        };
        let mime_type = if ext == "webm" { "audio/webm" } else { "audio/mp4" };

        let relative_audio_path = format!("audio/{video_id}.{ext}");
        let final_audio_path = self.base_dir.join(&relative_audio_path);
        let tmp_audio_path = self.base_dir.join(format!("audio/{video_id}.{ext}.tmp"));

        // Step 1: Stream bytes chunk-by-chunk to .tmp file
        let res = client
            .get(stream_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await
            .map_err(|e| format!("Failed to connect to stream CDN: {e}"))?;

        if !res.status().is_success() {
            return Err(format!("Stream CDN returned error status: {}", res.status()));
        }

        let mut file = File::create(&tmp_audio_path)
            .map_err(|e| format!("Failed to create temporary file {tmp_audio_path:?}: {e}"))?;

        let mut res = res;
        let mut total_bytes = 0u64;

        while let Some(chunk) = res.chunk().await.map_err(|e| format!("Chunk read error: {e}"))? {
            file.write_all(&chunk)
                .map_err(|e| format!("Write chunk error: {e}"))?;
            total_bytes += chunk.len() as u64;
        }

        file.flush().map_err(|e| format!("Flush error: {e}"))?;
        drop(file);

        // Step 2: Atomic rename .tmp -> final audio file
        fs::rename(&tmp_audio_path, &final_audio_path).map_err(|e| {
            format!("Atomic rename failed from {tmp_audio_path:?} to {final_audio_path:?}: {e}")
        })?;

        // Step 3: Best-effort artwork download
        let mut relative_artwork_path = None;
        if let Some(ref art_url) = track.artwork_url {
            let rel_art = format!("artwork/{video_id}.jpg");
            let full_art = self.base_dir.join(&rel_art);
            if let Ok(art_res) = client.get(art_url).send().await {
                if let Ok(bytes) = art_res.bytes().await {
                    if fs::write(&full_art, bytes).is_ok() {
                        relative_artwork_path = Some(rel_art);
                    }
                }
            }
        }

        // Step 4: Update metadata index
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = DownloadEntry {
            track,
            relative_audio_path,
            relative_artwork_path,
            file_size_bytes: total_bytes,
            mime_type: mime_type.to_string(),
            downloaded_at: now,
        };

        self.index.entries.insert(video_id.clone(), entry);
        self.save_index_atomic()?;

        tracing::info!(target: "app::download_manager", "Successfully downloaded track {video_id} ({total_bytes} bytes)");
        Ok(final_audio_path)
    }

    /// Deletes a downloaded track from disk and updates the index.
    pub fn delete_download(&mut self, video_id: &str) -> Result<(), String> {
        if let Some(entry) = self.index.entries.remove(video_id) {
            let audio_path = self.base_dir.join(&entry.relative_audio_path);
            if audio_path.exists() {
                let _ = fs::remove_file(audio_path);
            }
            if let Some(ref rel_art) = entry.relative_artwork_path {
                let art_path = self.base_dir.join(rel_art);
                if art_path.exists() {
                    let _ = fs::remove_file(art_path);
                }
            }
            self.save_index_atomic()?;
            Ok(())
        } else {
            Err(format!("Track {video_id} is not in download index"))
        }
    }

    /// Atomically persists `metadata.json` using `.tmp` rename.
    fn save_index_atomic(&self) -> Result<(), String> {
        let metadata_path = self.base_dir.join("metadata.json");
        let tmp_path = self.base_dir.join("metadata.json.tmp");

        let bytes = serde_json::to_vec_pretty(&self.index)
            .map_err(|e| format!("Failed to serialize download index: {e}"))?;

        fs::write(&tmp_path, bytes)
            .map_err(|e| format!("Failed to write metadata tmp file: {e}"))?;

        fs::rename(&tmp_path, &metadata_path)
            .map_err(|e| format!("Failed to rename metadata file: {e}"))?;

        Ok(())
    }
}
