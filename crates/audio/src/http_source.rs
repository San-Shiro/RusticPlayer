//! Ranged, seekable, range-buffered HTTP [`MediaSource`] for symphonia.
//!
//! Symphonia decodes synchronously on audio threads and, for container formats
//! like MP4/AAC (m4a), it *seeks*: it reads the container header at byte 0, then
//! jumps toward the end of the file (`SeekFrom::End(-X)`) to read the `moov`
//! atom metadata, and finally seeks back to the start to read audio frames.
//! That access pattern makes three properties non-negotiable for a network
//! source: cheap seeking, backward-seek tolerance, and content-length discovery.
//!
//! This implementation delivers all three without paying the catastrophic cost
//! of the previous design (which built a brand-new Tokio runtime on *every*
//! read):
//!
//! * **Shared transport.** A single process-wide [`reqwest::blocking::Client`]
//!   (with connection pooling) is reused by every `HttpSource`. The audio
//!   thread never constructs a runtime per read — the blocking client owns one
//!   background runtime for the whole process.
//! * **One-round-trip length discovery.** Instead of a `HEAD` request (which
//!   YouTube stream URLs frequently reject with 403/405), the initial probe is
//!   a `GET` with `Range: bytes=0-{CHUNK_SIZE-1}`. The `Content-Range:
//!   bytes 0-X/TOTAL` response header yields the exact total length in a single
//!   round trip, and the returned bytes are cached so none are wasted.
//! * **Sliding chunk window.** An in-memory LRU of [`CACHE_CHUNKS`] blocks
//!   (each [`CHUNK_SIZE`] bytes) means successive reads and small backward
//!   seeks — exactly the header/`moov`/back-to-start dance — are served from
//!   RAM instead of the network.
//!
//! An optional write-through `on_chunk` callback lets a disk cache observe
//! every freshly fetched block.

use std::io::{self, Read, Seek, SeekFrom};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use lru::LruCache;
use reqwest::StatusCode;
use reqwest::blocking::Client;
use reqwest::header::{CONTENT_RANGE, HeaderMap, RANGE};

use crate::error::{AudioError, Result};
use crate::source::{CHUNK_SIZE, MediaSource};

/// Number of [`CHUNK_SIZE`] blocks kept resident in the in-memory sliding
/// window. `32 * 64 KiB = 2 MiB` — comfortably covers symphonia's
/// header → `moov` → back-to-start seeks plus small backward scrubs without
/// re-hitting the network.
pub const CACHE_CHUNKS: usize = 32;

/// Retry budget for a single range request against *transient* transport
/// errors (connection resets, timeouts). HTTP status failures are surfaced
/// immediately without retry.
const MAX_RETRIES: usize = 3;

pub const YTM_BROWSER_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";

/// Process-wide blocking HTTP client.
///
/// `reqwest::blocking::Client` owns a single background Tokio runtime and a
/// connection pool; cloning it is a cheap `Arc` bump that shares that pool.
/// Sharing one client across every `HttpSource` keeps keep-alive connections
/// warm and guarantees the audio thread never spins up a runtime per read.
fn shared_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .user_agent(YTM_BROWSER_USER_AGENT)
            .pool_max_idle_per_host(8)
            .pool_idle_timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(12))
            .build()
            .expect("build shared reqwest blocking client")
    })
}

/// The raw result of a single range request, before it is sliced into a block.
struct RangeResponse {
    /// The response body bytes.
    body: Vec<u8>,
    /// The total resource length, if the server disclosed it.
    total: Option<u64>,
    /// `true` when the server returned `200 OK` (ignoring `Range`) and `body`
    /// therefore holds the *entire* resource starting at byte 0.
    is_full: bool,
}

/// Block-device view over a ranged HTTP URL.
///
/// Reads are served in [`CHUNK_SIZE`] windows. On first touch of a block we
/// issue a `Range: bytes=start-end` request; results are cached in a bounded
/// LRU and, optionally, written through to a disk cache via `on_chunk`.
///
/// Retry policy: exponential backoff (50ms, 100ms) on transient transport
/// errors; HTTP 4xx/5xx (other than `416 Range Not Satisfiable`) are returned
/// immediately as failures.
pub struct HttpSource {
    url: String,
    client: Client,
    /// Total byte length, discovered from `Content-Range` on the first fetch.
    total_len: Arc<Mutex<Option<u64>>>,
    /// Current logical position (bytes) for `Read`/`Seek`.
    pos: u64,
    /// Sliding window of recently fetched blocks, keyed by block index.
    cache: Arc<Mutex<LruCache<u64, Arc<[u8]>>>>,
    /// Write-through callback invoked with `(block_index, bytes)` for each
    /// block fetched from the network.
    on_chunk: Option<Box<dyn Fn(u64, &[u8]) + Send + Sync>>,
}

impl HttpSource {
    /// Create a source for `url`, with no write-through callback.
    pub fn new(url: impl Into<String>) -> Self {
        Self::with_callback(url, None)
    }

    /// Create a source for `url`, attaching a callback invoked with
    /// `(block_index, bytes)` for each block fetched from the network. Used by
    /// the `stream_cache` write-through layer.
    pub fn with_callback(
        url: impl Into<String>,
        on_chunk: Option<Box<dyn Fn(u64, &[u8]) + Send + Sync>>,
    ) -> Self {
        Self {
            url: url.into(),
            client: shared_client().clone(),
            total_len: Arc::new(Mutex::new(None)),
            pos: 0,
            cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(CACHE_CHUNKS).expect("CACHE_CHUNKS is non-zero"),
            ))),
            on_chunk,
        }
    }

    /// Fetch one block, serving from the LRU when possible.
    fn fetch_chunk(&self, index: u64) -> Result<Arc<[u8]>> {
        // Serve from the in-memory window first — covers back-to-back reads
        // into the same block and small backward seeks.
        if let Some(bytes) = self.cache.lock().unwrap().get(&index) {
            return Ok(bytes.clone());
        }

        let start = index * CHUNK_SIZE;

        // If we already know the total length and this block begins at or after
        // EOF, there is nothing to fetch.
        if let Some(total) = *self.total_len.lock().unwrap() {
            if start >= total {
                return Ok(empty_chunk());
            }
        }

        let end = start + CHUNK_SIZE - 1;
        let resp = self.http_get_range(start, end)?;

        // Learn (once) the authoritative total length. A 206 discloses it via
        // Content-Range; a 200 means the whole body was returned, so its length
        // is the total.
        let discovered = resp.total.or({
            if resp.is_full {
                Some(resp.body.len() as u64)
            } else {
                None
            }
        });
        if let Some(t) = discovered {
            let mut guard = self.total_len.lock().unwrap();
            if guard.is_none() {
                *guard = Some(t);
            }
        }

        // Extract this block's window. A 206 already returns exactly the window;
        // a 200 (server ignored Range) returns the whole resource from byte 0,
        // so we slice out the block ourselves.
        let chunk: Arc<[u8]> = if resp.is_full {
            let s = start as usize;
            if s >= resp.body.len() {
                empty_chunk()
            } else {
                let e = (s + CHUNK_SIZE as usize).min(resp.body.len());
                Arc::from(&resp.body[s..e])
            }
        } else {
            Arc::from(resp.body)
        };

        self.cache.lock().unwrap().put(index, chunk.clone());
        if let Some(cb) = &self.on_chunk {
            cb(index, &chunk);
        }
        Ok(chunk)
    }

    /// Issue a single ranged `GET` with retry+backoff on transient errors.
    fn http_get_range(&self, start: u64, end: u64) -> Result<RangeResponse> {
        let mut backoff = Duration::from_millis(50);
        let mut last_err: Option<AudioError> = None;

        for attempt in 0..MAX_RETRIES {
            let result = self
                .client
                .get(&self.url)
                .header(RANGE, format!("bytes={start}-{end}"))
                .header("User-Agent", YTM_BROWSER_USER_AGENT)
                .header("Referer", "https://music.youtube.com/")
                .header("Origin", "https://music.youtube.com")
                .send();

            match result {
                Ok(resp) => {
                    let status = resp.status();

                    // Past EOF: not an error — report an empty body, but still
                    // try to learn the total from `Content-Range: bytes */TOTAL`.
                    if status == StatusCode::RANGE_NOT_SATISFIABLE {
                        let total = parse_content_range_total(resp.headers());
                        return Ok(RangeResponse {
                            body: Vec::new(),
                            total,
                            is_full: false,
                        });
                    }

                    // Any other non-success status is a hard failure; do not
                    // retry 4xx/5xx (the URL is likely expired or invalid).
                    if !status.is_success() {
                        return Err(AudioError::Network(format!(
                            "http {status} for range {start}-{end}"
                        )));
                    }

                    // 200 means the server ignored our Range and returned the
                    // whole resource; 206 is a proper partial response.
                    let is_full = status == StatusCode::OK;
                    let total = parse_content_range_total(resp.headers());
                    let body = resp.bytes().map_err(AudioError::from)?.to_vec();
                    return Ok(RangeResponse {
                        body,
                        total,
                        is_full,
                    });
                }
                Err(e) => {
                    tracing::debug!(attempt, error = %e, "http range retry");
                    last_err = Some(e.into());
                    if attempt + 1 < MAX_RETRIES {
                        std::thread::sleep(backoff);
                        backoff *= 2;
                    }
                }
            }
        }

        Err(last_err.unwrap_or_else(|| AudioError::Network("range request failed".into())))
    }

    /// Discover the total content length, probing the origin if necessary.
    ///
    /// The probe is a single ranged `GET` for block 0. Its `Content-Range`
    /// header yields the total size, and the bytes are cached so the imminent
    /// read of the container header costs nothing more.
    fn discover_len(&self) -> Result<u64> {
        if let Some(l) = *self.total_len.lock().unwrap() {
            return Ok(l);
        }
        let _ = self.fetch_chunk(0)?;
        (*self.total_len.lock().unwrap())
            .ok_or_else(|| AudioError::Network("server did not disclose content length".into()))
    }
}

impl Read for HttpSource {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let index = self.pos / CHUNK_SIZE;
        let offset = (self.pos % CHUNK_SIZE) as usize;

        let bytes = self.fetch_chunk(index).map_err(io::Error::other)?;

        if offset >= bytes.len() {
            // Past the end of available data for this block -> EOF.
            return Ok(0);
        }

        let n = (bytes.len() - offset).min(buf.len());
        buf[..n].copy_from_slice(&bytes[offset..offset + n]);
        self.pos += n as u64;
        Ok(n)
    }
}

impl Seek for HttpSource {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos: i64 = match pos {
            SeekFrom::Start(n) => n as i64,
            SeekFrom::End(n) => {
                let len = self.discover_len().map_err(io::Error::other)? as i64;
                len + n
            }
            SeekFrom::Current(n) => self.pos as i64 + n,
        };

        if new_pos < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("negative seek target {new_pos}"),
            ));
        }

        self.pos = new_pos as u64;
        Ok(self.pos)
    }
}

impl MediaSource for HttpSource {
    fn len(&self) -> Option<u64> {
        *self.total_len.lock().unwrap()
    }

    fn is_remote(&self) -> bool {
        true
    }
}

/// A shared, zero-length block.
fn empty_chunk() -> Arc<[u8]> {
    Arc::from(Vec::<u8>::new())
}

/// Parse the total length out of a `Content-Range` header.
///
/// Accepts both `bytes 0-63/12345` (partial content) and `bytes */12345`
/// (range-not-satisfiable) forms. Returns `None` when the header is absent,
/// malformed, or the total is the unknown sentinel `*`.
fn parse_content_range_total(headers: &HeaderMap) -> Option<u64> {
    let value = headers.get(CONTENT_RANGE)?.to_str().ok()?;
    let slash = value.rfind('/')?;
    let total = value[slash + 1..].trim();
    if total == "*" {
        return None;
    }
    total.parse::<u64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Deterministic test payload spanning several blocks (3 full + a partial).
    fn test_data(len: usize) -> Vec<u8> {
        (0..len).map(|i| (i % 251) as u8).collect()
    }

    /// Parse a `bytes=start-end` Range header value into an inclusive pair.
    fn parse_range(value: &str, total: u64) -> (u64, u64) {
        let value = value.trim();
        let spec = value.strip_prefix("bytes=").unwrap_or(value);
        let mut parts = spec.splitn(2, '-');
        let start = parts
            .next()
            .unwrap_or("0")
            .trim()
            .parse::<u64>()
            .unwrap_or(0);
        let end_raw = parts.next().unwrap_or("").trim();
        let end = if end_raw.is_empty() {
            total.saturating_sub(1)
        } else {
            end_raw.parse::<u64>().unwrap_or(total.saturating_sub(1))
        };
        (start, end)
    }

    /// A minimal HTTP/1.1 server that honours `Range` requests and, optionally,
    /// rejects `HEAD` (mimicking YouTube stream URLs). Returns the base URL and
    /// a counter of how many requests it served.
    fn spawn_range_server(data: Vec<u8>, reject_head: bool) -> (String, Arc<AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let addr = listener.local_addr().expect("local addr");
        let hits = Arc::new(AtomicUsize::new(0));
        let hits_thread = hits.clone();

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = match stream {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                // Read the request head (until the blank line).
                let mut reader = match stream.try_clone() {
                    Ok(s) => io::BufReader::new(s),
                    Err(_) => continue,
                };
                let mut request_line = String::new();
                if io::BufRead::read_line(&mut reader, &mut request_line).is_err() {
                    continue;
                }
                let method = request_line
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();

                let mut range_header: Option<String> = None;
                loop {
                    let mut line = String::new();
                    match io::BufRead::read_line(&mut reader, &mut line) {
                        Ok(0) => break,
                        Ok(_) => {}
                        Err(_) => break,
                    }
                    let trimmed = line.trim_end();
                    if trimmed.is_empty() {
                        break;
                    }
                    let lower = trimmed.to_ascii_lowercase();
                    if let Some(rest) = lower.strip_prefix("range:") {
                        range_header = Some(rest.trim().to_string());
                    }
                }

                hits_thread.fetch_add(1, Ordering::SeqCst);
                let total = data.len() as u64;

                if method == "HEAD" && reject_head {
                    let resp = "HTTP/1.1 405 Method Not Allowed\r\n\
                                Content-Length: 0\r\nConnection: close\r\n\r\n";
                    let _ = stream.write_all(resp.as_bytes());
                    continue;
                }

                let (start, end) = match &range_header {
                    Some(r) => parse_range(r, total),
                    None => (0, total.saturating_sub(1)),
                };

                if start >= total {
                    let resp = format!(
                        "HTTP/1.1 416 Range Not Satisfiable\r\n\
                         Content-Range: bytes */{total}\r\n\
                         Content-Length: 0\r\nConnection: close\r\n\r\n"
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    continue;
                }

                let end = end.min(total - 1);
                let slice = &data[start as usize..=end as usize];
                let header = format!(
                    "HTTP/1.1 206 Partial Content\r\n\
                     Content-Range: bytes {start}-{end}/{total}\r\n\
                     Content-Length: {}\r\n\
                     Accept-Ranges: bytes\r\nConnection: close\r\n\r\n",
                    slice.len()
                );
                let _ = stream.write_all(header.as_bytes());
                let _ = stream.write_all(slice);
                let _ = stream.flush();
            }
        });

        (format!("http://{addr}/audio"), hits)
    }

    #[test]
    fn content_length_via_content_range() {
        let len = CHUNK_SIZE as usize * 3 + 1234;
        // reject_head = true: prove we never rely on HEAD for length.
        let (url, hits) = spawn_range_server(test_data(len), true);

        let mut src = HttpSource::new(url);
        assert_eq!(MediaSource::len(&src), None, "length unknown before probe");

        // End-relative seek forces content-length discovery via one ranged GET.
        let end_pos = src.seek(SeekFrom::End(0)).expect("seek to end");
        assert_eq!(end_pos, len as u64);
        assert_eq!(MediaSource::len(&src), Some(len as u64));

        // Discovery must cost exactly one request (the block-0 probe), no HEAD.
        assert_eq!(hits.load(Ordering::SeqCst), 1, "one round trip to discover length");
    }

    #[test]
    fn sequential_reads_across_block_boundary() {
        let len = CHUNK_SIZE as usize * 3 + 1234;
        let data = test_data(len);
        let (url, _hits) = spawn_range_server(data.clone(), false);

        let mut src = HttpSource::new(url);
        let mut got = Vec::new();
        src.read_to_end(&mut got).expect("read to end");

        assert_eq!(got.len(), len, "read the whole resource");
        assert_eq!(got, data, "bytes match across block boundaries");
    }

    #[test]
    fn backward_seek_serves_moov_pattern() {
        // Mimic symphonia's MP4 flow: read header at 0, seek near the end for
        // the moov atom, then seek back to the start.
        let len = CHUNK_SIZE as usize * 3 + 1234;
        let data = test_data(len);
        let (url, _hits) = spawn_range_server(data.clone(), true);

        let mut src = HttpSource::new(url);

        // 1. Read the container "header" at byte 0.
        let mut header = [0u8; 16];
        src.read_exact(&mut header).expect("read header");
        assert_eq!(&header, &data[..16]);

        // 2. Seek toward the end (SeekFrom::End(-X)) and read the "moov" tail.
        let tail_len = 32usize;
        src.seek(SeekFrom::End(-(tail_len as i64))).expect("seek end");
        let mut tail = vec![0u8; tail_len];
        src.read_exact(&mut tail).expect("read tail");
        assert_eq!(tail, data[len - tail_len..]);

        // 3. Seek back to the start; block 0 is still cached.
        src.seek(SeekFrom::Start(0)).expect("seek start");
        let mut header2 = [0u8; 16];
        src.read_exact(&mut header2).expect("re-read header");
        assert_eq!(&header2, &data[..16]);
    }

    #[test]
    fn out_of_bounds_and_negative_seeks() {
        let len = CHUNK_SIZE as usize * 2;
        let data = test_data(len);
        let (url, _hits) = spawn_range_server(data, false);

        let mut src = HttpSource::new(url);

        // Seeking past EOF is allowed; the subsequent read reports EOF (0).
        let pos = src
            .seek(SeekFrom::Start(len as u64 + 4096))
            .expect("seek past eof");
        assert_eq!(pos, len as u64 + 4096);
        let mut buf = [0u8; 32];
        assert_eq!(src.read(&mut buf).expect("read past eof"), 0);

        // A negative absolute target is an error.
        let err = src
            .seek(SeekFrom::End(-((len as i64) + 1)))
            .expect_err("negative target must fail");
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);

        // Current-relative underflow is likewise an error.
        src.seek(SeekFrom::Start(10)).expect("seek to 10");
        let err = src
            .seek(SeekFrom::Current(-100))
            .expect_err("current underflow must fail");
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn write_through_callback_observes_every_block() {
        let len = CHUNK_SIZE as usize * 3 + 1234;
        let data = test_data(len);
        let (url, _hits) = spawn_range_server(data.clone(), false);

        let seen: Arc<Mutex<Vec<(u64, usize)>>> = Arc::new(Mutex::new(Vec::new()));
        let seen_cb = seen.clone();
        let mut src = HttpSource::with_callback(
            url,
            Some(Box::new(move |index, bytes| {
                seen_cb.lock().unwrap().push((index, bytes.len()));
            })),
        );

        let mut got = Vec::new();
        src.read_to_end(&mut got).expect("read to end");
        assert_eq!(got, data);

        let seen = seen.lock().unwrap();
        // Four blocks: 0,1,2 full + 3 partial.
        assert_eq!(seen.len(), 4, "callback fired once per fetched block");
        assert_eq!(seen[0], (0, CHUNK_SIZE as usize));
        assert_eq!(seen[3], (3, 1234));
    }
}
