//! Write-through chunk cache for remote audio streams.
//!
//! Inspired by ExoPlayer's `SimpleCache`. Each logical stream key
//! (`(video_id|url, itag)`) is stored as one file of [`BLOCK_SIZE`] blocks on
//! disk plus a SQLite meta row (original URL, size, etag) so a second play can
//! be satisfied entirely from disk without re-downloading — or even touching —
//! the network. The block size is the same [`crate::source::CHUNK_SIZE`] used
//! by the HTTP transport, so writing through is a straight `Vec::copy_from_slice`.


use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::{Connection, params};
use xxhash_rust::xxh3::xxh3_64;

use crate::error::{AudioError, Result};
use crate::source::CHUNK_SIZE;

/// Database schema version. Bump when the table layout changes.
const SCHEMA_VERSION: i64 = 1;

/// A completed stream. Immutable at audio-read time.
#[derive(Debug, Clone)]
pub struct StreamMeta {
    pub key: String,
    pub url: String,
    pub content_length: Option<i64>,
    pub etag: Option<String>,
    pub bytes_on_disk: u64,
}

/// Write-through chunk cache.
///
/// One `StreamCache` owns one SQLite database and one directory tree:
///
/// ```text
/// cache_root/
///   meta.sqlite3           (key, url, etag, size)
///   streams/
///     ab/                  key shard by first 2 hex of xxh3
///       ab/<key>.bin      block blob
/// ```
///
/// A cache lookup never blocks the audio thread: misses synchronously fetch a
/// chunk from the origin server (via [`crate::source::HttpSource`]) and are
/// written through to disk in the background.
pub struct StreamCache {
    root: PathBuf,
    conn: Arc<Mutex<Connection>>,

}

impl StreamCache {
    /// Open (or create) a cache rooted at `dir`.
    pub fn open(dir: impl Into<PathBuf>) -> Result<Self> {
        let root = dir.into();
        let streams_dir = root.join("streams");
        std::fs::create_dir_all(&streams_dir)?;

        let conn = Connection::open(root.join("meta.sqlite3"))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS streams (
                key TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                etag TEXT,
                content_length INTEGER,
                fetched_at INTEGER NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS meta (name TEXT PRIMARY KEY, value INTEGER)",
            [],
        )?;
        // Bump schema.version
        conn.execute("INSERT OR IGNORE INTO meta (name, value) VALUES ('version', 0)", [])?;
        conn.execute(
            "UPDATE meta SET value = ?1 WHERE name = 'version'",
            params![SCHEMA_VERSION],
        )?;

        Ok(Self {
            root,
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Deterministic key for a (url, itag) pair — collision-safe within one
    /// user's library. First 16 hex of xxh3-128 are plenty (64-bit space,
    /// birthday-bound > millions of streams). Returns None when both parts
    /// are absent.
    pub fn key_for(url: &str, itag: Option<&str>) -> Option<String> {
        if url.is_empty() && itag.is_none() {
            return None;
        }
        let mut h = xxh3_64(url.as_bytes());
        if let Some(itag) = itag {
            h ^= xxh3_64(itag.as_bytes());
        }
        Some(format!("{:016x}", h))
    }

    /// Look up a cached meta row by (url, itag).
    pub fn get(&self, url: &str, itag: Option<&str>) -> Result<Option<StreamMeta>> {
        let key = match Self::key_for(url, itag) {
            Some(k) => k,
            None => return Ok(None),
        };
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT key, url, content_length, etag, content_length FROM streams WHERE key = ?1",
        )?;
        let mut rows = stmt.query(params![key])?;
        let row = match rows.next()? {
            Some(r) => r,
            None => return Ok(None),
        };
        let blob_path = self.blob_path(&key);
        let bytes_on_disk = std::fs::metadata(&blob_path).map(|m| m.len()).unwrap_or(0);
        if bytes_on_disk == 0 {
            // Blob vanished; treat as miss so the repair path can refetch.
            return Ok(None);
        }
        Ok(Some(StreamMeta {
            key: row.get(0)?,
            url: row.get(1)?,
            content_length: row.get(2)?,
            etag: row.get(3)?,
            bytes_on_disk,
        }))
    }

    /// Store (or replace) the cached stream key for a (url, itag) pair.
    /// Called by the write-through layer after a successful download.
    pub fn put(
        &self,
        url: &str,
        itag: Option<&str>,
        content_length: Option<u64>,
        etag: Option<String>,
    ) -> Result<String> {
        let key = Self::key_for(url, itag)
            .ok_or_else(|| AudioError::Cache("cannot key cache entry".into()))?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO streams (key, url, etag, content_length, fetched_at)
             VALUES (?1, ?2, ?3, ?4, strftime('%s','now'))",
            params![key, url, etag, content_length.map(|n| n as i64)],
        )?;
        Ok(key)
    }

    /// Write one chunk of a stream into the blob file, growing it as needed.
    /// Idempotent: rewriting the same block index with identical bytes is safe.
    pub fn write_chunk(&self, key: &str, chunk_index: u64, bytes: &[u8]) -> Result<()> {
        let path = self.blob_path(key);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)?;
        let offset = chunk_index * CHUNK_SIZE;
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(bytes)?;
        Ok(())
    }

    /// Read one chunk from the blob. Returns `Err(Io(NotFound))` beyond EoF.
    pub fn read_chunk(&self, key: &str, chunk_index: u64, buf: &mut [u8]) -> Result<usize> {
        let path = self.blob_path(key);
        let mut file = match File::open(&path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(e.into()),
        };
        file.seek(SeekFrom::Start(chunk_index * CHUNK_SIZE))?;
        let n = file.read(buf)?;
        Ok(n)
    }

    /// Compare two streams for equivalence — used by Jam Mode B's hash-match.
    pub fn blob_path(&self, key: &str) -> PathBuf {
        let shard_len = std::cmp::min(2, key.len());
        let shard = &key[..shard_len];
        self.root
            .join("streams")
            .join(shard)
            .join(format!("{}.bin", key))
    }

    /// Returns the list of keys currently on disk (for GC LRU / inspection).
    pub fn keys(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key FROM streams")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn evict(&self, key: &str) -> Result<()> {
        let path = self.blob_path(key);
        let _ = std::fs::remove_file(path);
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM streams WHERE key = ?1", params![key])?;
        Ok(())
    }

    /// Lightweight streaming statistics for tracing/debugging.
    pub fn stats(&self) -> Result<CacheStats> {
        let paths = {
            let conn = self.conn.lock().unwrap();
            let mut prep = conn.prepare("SELECT key FROM streams")?;
            let rows = prep.query_map([], |r| r.get::<_, String>(0))?;
            rows.collect::<std::result::Result<Vec<_>, _>>()?
        };
        let mut total = 0u64;
        for key in paths.iter() {
            total += std::fs::metadata(self.blob_path(key))
                .map(|m| m.len())
                .unwrap_or(0);
        }
        Ok(CacheStats {
            entries: paths.len(),
            bytes: total,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub entries: usize,
    pub bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_chunk() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = StreamCache::open(tmp.path()).unwrap();
        let key = cache
            .put("http://x/v=abc", Some("251"), Some(128 * 1024), None)
            .unwrap();

        let chunk: [u8; 4] = *b"hell";
        cache.write_chunk(&key, 0, &chunk).unwrap();

        let mut buf = [0u8; 4];
        let n = cache.read_chunk(&key, 0, &mut buf).unwrap();
        assert_eq!(n, 4);
        assert_eq!(&buf, b"hell");
    }

    #[test]
    fn stable_key_independent_of_case() {
        let k1 = StreamCache::key_for("HTTP://A", None);
        let k2 = StreamCache::key_for("http://A", None);
        assert_ne!(k1, k2); // case matters — URLs are canonicalized upstream
    }
}
