# Rustic Player — Revised Phase 00 Implementation Plan

Revised specification for **Phase 00 — Contracts & Foundation**, incorporating the review fixes against the original plan. Everything after this phase is additive-only, versioned by `API_VERSION` (milestone **S0** — contract freeze).

## Changes from the original plan

1. **Contract types moved to `crates/core`** — the original plan had `sources`/`jam` importing `engine::api` while `engine` imports `sources`/`jam` (a Cargo dependency cycle). The frozen contract now lives in `core::api`; `engine` re-exports it so `engine::api::Track` paths keep working.
2. **`crates/plugin-api` removed** — it was an orphan stub; the contract now lives in `core`.
3. **Dependency test relocated** — from root `tests/` (ignored by Cargo's virtual workspace manifest) to `crates/engine/tests/`, using `cargo_metadata`.
4. **Typed errors in frozen traits** — `Result<_, String>` replaced with `thiserror` enums (`SourceError`, `JamError`, `EngineError.kind`), since trait signatures freeze at S0.
5. **Trait signatures reconciled with `plans/core-engine-plan.md` §4** — `resolve(&Track)`, `library() -> Vec<Playlist>`, `AuthFlow`/`AuthState` types added to the contract.
6. **Self-contained contract listing** — kept types (`Command` wrapper, `PlayContext`, `RepeatMode`, `PlaybackState`, `SearchResults`) are restated in full.
7. **Restored missing phase-doc tasks** — `profiling` feature → tracy, gpui pin for Track B, CI on 3 OSes.

---

## Architecture Overview & Dependency Rules

```
crates/app (GPUI UI) ────────┐
                             ▼
crates/cli ───────────► crates/engine ─────────► crates/core (frozen Command/Event contract, domain models, queue, matching)
                            │
                            ├──► crates/audio    (Symphonia decode, cpal output, rtrb ring, rubato resample)
                            ├──► crates/library  (rusqlite FTS5, scanner, artwork, peak waveform)
                            ├──► crates/sources  (YTM Innertube, Spotify rspotify, resolver) ──┐
                            ├──► crates/services (LRCLIB, SponsorBlock, Last.fm, Discord RPC)  ├─ depend on crates/core ONLY
                            ├──► crates/platform (MPRIS, SMTC, media keys, single instance)    │  (never on engine, never on each other)
                            ├──► crates/jam      (Host-authoritative session, LAN WS, WebRTC) ─┘
                            └──► crates/ui-kit   (Pure UI components & design tokens — GPUI only)
```

### Dependency Invariants
1. `crates/core` MUST NOT import any other workspace crate. It owns the frozen contract (`core::api`).
2. `crates/sources`, `crates/services`, `crates/audio`, `crates/library`, `crates/platform`, `crates/jam`, and `crates/ui-kit` MUST NOT cross-import each other. Among workspace crates, `sources`/`services`/`jam`/`platform` may depend on `crates/core` only.
3. `crates/engine` is the single orchestrator — the only crate importing the lower-level engine crates. It re-exports the contract (`pub use core::api::*` in `engine/src/api.rs`).
4. `crates/ui-kit` depends only on GPUI and rendering primitives — no `engine`, no `core`.
5. `crates/plugin-api` is deleted; the contract lives in `core`.

---

## Phase 00 — Contracts & Foundation (CURRENT PHASE)

### Goal
Freeze the `Command`/`Event` protocol that UI (Track B) and Engine (Track A) build against for the entire project lifecycle. Create workspace crate skeletons and trait boundaries, set up `tracing`, and generate `plans/engine-api-contract.md`.

---

### Task 1 — Workspace Configuration
- **File**: `Cargo.toml`
- Members (existing 8 minus `plugin-api`, plus 4 new):
  ```toml
  [workspace]
  resolver = "2"
  members = [
      "crates/core",
      "crates/audio",
      "crates/library",
      "crates/platform",
      "crates/engine",
      "crates/cli",
      "crates/app",
      "crates/sources",
      "crates/jam",
      "crates/services",
      "crates/ui-kit",
  ]
  ```
- Delete `crates/plugin-api/` (superseded by `core::api`).
- Delete the empty orphaned root `src/` directory.
- Record the pinned gpui SHA in the workspace manifest (Track B prep, see Task 3-D).

---

### Task 2 — Frozen Contract (`crates/core/src/api.rs`)
Move the contract out of `engine` into `core`. The listing below is **self-contained and complete** — it restates the kept pre-freeze types and the new model types together.

- **File**: `crates/core/src/api.rs`
- Add to `crates/core/Cargo.toml`: `schemars = "0.8"` (pinned — 0.8 guarantees `JsonSchema` impls for `Duration`/`PathBuf`; re-verify if upgrading to 1.x).

```rust
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::path::PathBuf;
use std::time::Duration;

pub type TrackId = String;
pub type PeerId = String;

pub const API_VERSION: &str = "1.0.0";
pub const PROTOCOL_VERSION: u32 = 1;

// ---- playback primitives (kept from pre-freeze api.rs) ----

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum PlayContext {
    Album(String),
    Artist(String),
    Playlist(String),
    AllTracks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Buffering,
}

// ---- tracks & sources ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum TrackSource {
    Local(PathBuf),
    YouTubeMusic { video_id: String },
    Spotify { spotify_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Track {
    pub id: TrackId, // "src:provider:id"
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub duration_secs: u32,
    pub source: TrackSource,
    pub isrc: Option<String>,
    pub artwork_url: Option<String>,
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Playlist {
    pub id: String,
    pub provider: String, // "local" | "ytmusic" | "spotify"
    pub name: String,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResolvedAudio {
    pub stream_url: String,
    pub codec: String,
    pub bitrate_kbps: u32,
    pub expires_at_unix_secs: Option<u64>,
}

// ---- auth ----

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AuthFlow {
    Pkce,
    BrowserCookie,
    AccessToken(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuthState {
    pub provider: String,
    pub logged_in: bool,
    pub account_name: Option<String>,
}

// ---- commands ----

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Command {
    pub origin: Option<PeerId>,
    pub action: CommandAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum CommandAction {
    // Transport
    Play,
    Pause,
    TogglePlay,
    Stop,
    Next,
    Previous,
    Seek(Duration),
    SetVolume(f32),

    // Resolution & Online Sources
    ResolveAndPlay(TrackId),
    Login { provider: String },
    ImportPlaylist { provider: String, external_id: String },

    // Queue
    PlayTrack(TrackId),
    PlayContext { context: PlayContext, start_at: usize },
    EnqueueNext(Vec<TrackId>),
    EnqueueEnd(Vec<TrackId>),
    RemoveFromQueue(usize),
    MoveInQueue { from: usize, to: usize },
    SetShuffle(bool),
    SetRepeat(RepeatMode),

    // Library
    ScanFolders(Vec<PathBuf>),
    RescanChanged,
    Search { query: String, limit: usize },
}

// ---- events ----

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchResults {
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ErrorKind {
    Network,
    Auth,
    Resolve,
    Decode,
    Library,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EngineError {
    pub kind: ErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Event {
    PlaybackStateChanged(PlaybackState),
    PositionChanged { position_secs: f64, duration_secs: f64 },
    TrackChanged(Option<Track>),
    Resolved { track_id: TrackId, audio: ResolvedAudio },
    VolumeChanged(f32),

    QueueChanged(Vec<Track>),
    ShuffleChanged(bool),
    RepeatChanged(RepeatMode),

    AuthStateChanged { provider: String, logged_in: bool },
    LyricsChanged { track_id: TrackId, lrc_content: Option<String> },
    JamSessionChanged { session_id: Option<String>, peers: Vec<PeerId> },

    ScanProgress { done: usize, total: usize },
    ScanComplete { added: usize, updated: usize, removed: usize },
    LibraryChanged,
    SearchResults { query: String, results: SearchResults },

    Error(EngineError),
}
```

- **File**: `crates/engine/src/api.rs` — replace contents with a re-export shim so existing `engine::api::*` paths keep compiling:
  ```rust
  pub use core::api::*;
  ```
- Add `core = { path = "../core" }` to `crates/engine/Cargo.toml`.

> **Breaking-change note (allowed now, forbidden after S0):** `Track` loses `artist`/`album_id`/`duration`/`src` in favor of `artists`/`album`/`duration_secs`/`source`+metadata; `PositionChanged` moves from `Duration` to `f64` seconds. Track B must sign off on these before freeze (see Verification).

---

### Task 3 — Crate Skeleton Definitions

#### A. Sources Crate (`crates/sources`)
- **Files**: `crates/sources/Cargo.toml`, `crates/sources/src/lib.rs`
- Depends on `core` only (+ `async-trait`, `thiserror`, `tracing`).
- Signatures reconciled with `plans/core-engine-plan.md` §4 (`resolve` takes `&Track`; `library` returns playlists; typed errors):

```rust
use async_trait::async_trait;
use core::api::{AuthFlow, AuthState, Playlist, ResolvedAudio, Track};

#[async_trait]
pub trait SourceProvider: Send + Sync {
    fn slug(&self) -> &'static str; // "ytmusic" | "spotify" | "local"
    async fn login(&self, flow: AuthFlow) -> Result<AuthState, SourceError>;
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<Track>, SourceError>;
    async fn library(&self) -> Result<Vec<Playlist>, SourceError>; // liked songs = synthetic playlist
    async fn resolve(&self, track: &Track) -> Result<ResolvedAudio, SourceError>; // local: trivial
}

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("not authenticated with {0}")]
    NotAuthenticated(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("track unavailable: {0}")]
    Unavailable(String),
}
```

#### B. Jam Session Crate (`crates/jam`)
- **Files**: `crates/jam/Cargo.toml`, `crates/jam/src/lib.rs`
- Depends on `core` only (+ `async-trait`, `thiserror`, `tracing`):

```rust
use async_trait::async_trait;
use core::api::Command;

#[async_trait]
pub trait JamTransport: Send + Sync {
    async fn connect(&mut self, endpoint: &str) -> Result<(), JamError>;
    async fn send(&mut self, cmd: &Command) -> Result<(), JamError>;
    async fn recv(&mut self) -> Result<Option<Command>, JamError>;
    async fn disconnect(&mut self) -> Result<(), JamError>;
}

#[derive(Debug, thiserror::Error)]
pub enum JamError {
    #[error("connection failed: {0}")]
    Connect(String),
    #[error("transport closed")]
    Closed,
    #[error("protocol error: {0}")]
    Protocol(String),
}
```

#### C. Services Crate (`crates/services`)
- **Files**: `crates/services/Cargo.toml`, `crates/services/src/lib.rs`
- Depends on `core` only. Stub modules (no network impls this phase):
  - `lyrics` (LRCLIB REST client stub)
  - `sponsorblock` (SponsorBlock segment client stub)
  - `scrobbler` (Last.fm scrobble client stub)
  - `discord` (Discord RPC client stub)

#### D. UI Kit Crate (`crates/ui-kit`) — Track B prep
- **Files**: `crates/ui-kit/Cargo.toml`, `crates/ui-kit/src/lib.rs`
- Pure layout, component, and design token definitions. Depends on gpui + gpui-component only.
- Pin gpui by git SHA in the workspace `Cargo.toml`; both `crates/ui-kit` and `crates/app` must compile against the pin (per `phase/phase-00-contracts.md` Track B prep).

---

### Task 4 — Architecture & Dependency Direction Test
- **File**: `crates/engine/tests/dependency_architecture_test.rs` (NOT root `tests/` — the workspace root is a virtual manifest, so root-level test targets are never built).
- Dev-dependency: `cargo_metadata` in `crates/engine/Cargo.toml` (use the resolved dependency graph; parsing `Cargo.toml` files misses transitive edges).
- Assertions:
  1. `core` has zero workspace dependencies.
  2. `sources`, `services`, `jam`, `audio`, `library`, `platform`, `ui-kit` have no edges to each other; `sources`/`services`/`jam`/`platform` depend on `core` only (among workspace crates).
  3. `ui-kit` has no edge to `engine` or `core`.
  4. `engine` is the only workspace crate depending on `audio`, `library`, `sources`, `services`, `platform`, `jam`.
- Run: `cargo test -p engine --test dependency_architecture_test`

---

### Task 5 — Schema Exporter & Contract Document
- **File**: `crates/engine/src/bin/export_contract.rs`
- Generate JSON schema for `Command` and `Event` via `schemars::schema_for!` (types resolve through the `engine::api` re-export).
- Write human-readable + embedded JSON Schema document to `plans/engine-api-contract.md`; commit it.

---

### Task 6 — Platform Foundation & Hygiene
- `tracing` + `tracing-subscriber` wired in the 4 new crates (already present in the existing 8).
- `profiling` cargo feature → tracy (per `phase/phase-00-contracts.md`).
- CI matrix: build / test / clippy `-D warnings` / fmt on Linux + Windows + macOS.

---

## Future Phase Roadmap & Technical Targets

| Phase | Title | Key Technical Deliverables | Verification Gate |
|---|---|---|---|
| **Phase 01** | Audio Core & Kit Scaffold | `MediaSource` (`FileSource`, `HttpSource`), `stream_cache`, Symphonia decoder, `cpal` + `rtrb` ring output (alloc-free callback), crossfade pipeline, `ui-kit` design tokens + gallery example. | Glitch-free local + ranged HTTP stream playback via CLI; UI gallery binary executes. |
| **Phase 02** | Library & Atoms/Molecules | `rusqlite` SQLite schema + FTS5 search, `jwalk` + `lofty` parallel indexer, WebP artwork pipeline, peak waveform cache; UI-Kit button/slider/card components. | 10k tracks scanned in <60s; rescan does zero I/O; UI components verified across states. |
| **Phase 03** | Sources & Organisms | Innertube YTM client (`browse`/`search`/`player`), `rspotify` PKCE client, ISRC resolution & scoring engine; UI Virtualized track lists & now-playing bar. | YTM/Spotify search -> resolve -> play gaplessly; track list renders 10,000 items with p99 frame time <8ms. |
| **Phase 04** | Engine Integration & Shell Pages | Mixed-source queue persistence, resolve-ahead scheduler, CLI daily driver; App shell router, MockEngine test harness, desktop page layouts. | **S1** MockEngine integration gate; **S2** CLI daily driver with 200-track soak test (RSS flat ±10%). |
| **Phase 05** | Services & Platform | LRCLIB sync lyrics parser, SponsorBlock auto-skip monitor, Last.fm scrobbler (50%/240s rule), MPRIS/SMTC platform media keys; UI live data binding. | Lyrics line-sync accuracy; SponsorBlock skips within 50ms; MPRIS media keys control player. |
| **Phase 06** | Jam Session & Panels | Host-authoritative session state machine, mDNS-SD + WebSocket LAN transport, clock offset compensation; Jam UI panel. | **S3** LAN Jam session syncs playback position across devices with <100ms drift. |
| **Phase 07** | Jam Transports & Polish | WebRTC P2P transport with E2E Noise encryption, Cast receiver fallback; motion design tokens, context menus, drag-and-drop. | P2P audio sync under packet loss; zero CPU when paused; RSS budget <150MB. |
| **Phase 08** | Hardening & Release | Automated error taxonomy handling, 8-hour soak test, CI release matrix for Windows, macOS, Linux binaries. | **S4** 8-hour continuous soak test; <150MB RSS; 0% idle CPU; 0 panics in test corpus. |

---

## Detailed Verification Plan for Phase 00

1. **Compilation & Linting**:
   - `cargo build --workspace`
   - `cargo test --workspace`
   - `cargo clippy --workspace -- -D warnings`
   - `cargo fmt --check`
   - CI green on Linux, Windows, macOS

2. **Dependency Architecture Verification**:
   - `cargo test -p engine --test dependency_architecture_test`

3. **Contract Export Verification**:
   - `cargo run -p engine --bin export_contract`
   - Verify `plans/engine-api-contract.md` is generated and committed with valid JSON schemas.

4. **Track B Sign-off (last chance before freeze)**:
   - UI track reads the frozen contract and flags anything the UI cannot express.
   - `crates/ui-kit` + `crates/app` compile against the pinned gpui SHA.

**Done when:** workspace builds on 3 OSes, dependency test passes, contract doc committed, Track B sign-off received — **S0 declared**.
