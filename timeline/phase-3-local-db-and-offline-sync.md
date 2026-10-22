# Phase 3: Local Database, Offline Sync & Google History

> **Sprint Goal**: Implement crash-consistent offline playback resolution (playing downloaded tracks directly from disk with zero network dependency), full SQLite metadata indexing with FTS5 search, and Google watchtime/history beacons.

---

## 1. Local Database & SQLite Architecture

### Step 3.1: SQLite Schema Upgrade & Performance Tuning (`crates/engine/src/db.rs`)
- [ ] **T3.1.1 — Production Schema Migration**:
  - Enforce WAL mode (`PRAGMA journal_mode = WAL`) and `PRAGMA synchronous = NORMAL` for non-blocking asynchronous reads/writes.
  - Create indexed tables for `tracks`, `playlists`, `playlist_tracks`, `artists`, `albums`, and `playback_history`.
- [ ] **T3.1.2 — Full-Text Search (FTS5)**:
  - Create SQLite `fts_tracks` virtual table covering title, artist, and album.
  - Implement sub-millisecond local search queries that match partial typos and prefixes.
- [ ] **T3.1.3 — Bounded Cache & Metadata Eviction**:
  - Automatically prune cached album artwork and unpinned track metadata when the user's disk cache threshold is reached.

---

## 2. Offline Sync & Local-First Playback Hot Path

### Step 3.2: Seamless Offline Playback Resolution (`crates/engine` & `crates/app`)
- [ ] **T3.2.1 — O(1) Offline Track Interception**:
  - In `EngineLoop::fetch_track` and `SourceResolver::resolve`:
    - Before making any network request to YouTube Music, query `DownloadManager`'s memory index for `downloads/audio/{video_id}.m4a` or `.webm`.
    - If present, seamlessly resolve to `TrackSource::Local(path)` and feed `FileSource` directly to Symphonia.
  - **Result**: Instant playback (<10ms latency) and 0 bytes transferred over the network for downloaded tracks.
- [ ] **T3.2.2 — Background Download Pipeline & Queue**:
  - Crash-consistent downloading: write stream blocks to `.tmp` file, verify checksum, and atomically rename upon completion.
  - Embed downloaded ID3/MP4 metadata tags and save 256x256 WebP album art to `downloads/artwork/{video_id}.webp`.
- [ ] **T3.2.3 — Dedicated "Downloads" View in Library Tab**:
  - Filter downloaded tracks, albums, and playlists.
  - Visual offline indicator badges throughout the app (checkmarks on tracks saved offline).

---

## 3. Google Playback & Watchtime History Beacon

### Step 3.3: Syncing Played Songs to YouTube History (`crates/sources/src/ytmusic`)
- [ ] **T3.3.1 — Parse PlaybackTracking in `PlayerResponse`**:
  - Extract `videostatsPlaybackUrl`, `videostatsWatchtimeUrl`, and `atrUrl` from Innertube's `/player` endpoint.
- [ ] **T3.3.2 — Playback Start Beacon**:
  - When a song starts playing, send an HTTP GET request to `videostatsPlaybackUrl` with:
    - Generated 16-character alphanumeric CPN (`Client Playback Nonce`).
    - `ver=2`, `c=WEB_REMIX`, and optional `list={playlist_id}` / referrer.
    - Authenticated with `SAPISIDHASH` + cookies.
  - **Result**: Track immediately shows up in the user's official YouTube Music History and "Listen Again" carousel.
- [ ] **T3.3.3 — Watchtime Periodic Flush**:
  - At 10s, 20s, and 50% track duration, ping `videostatsWatchtimeUrl` with elapsed media time (`cmt={secs}`) so YouTube's recommendation engine logs a full, genuine listen.

---

## 4. Verification & Acceptance Criteria

1. **True Airplane Mode**: Cut network connection entirely; any downloaded track plays instantly without hang, spinner, or error.
2. **Official History Verification**: Playing a song in RusticPlayer causes it to immediately show on [music.youtube.com](https://music.youtube.com) under History.
3. **Database Performance**: Querying a 10,000-track local library executes in <2ms with zero UI frame drops.
