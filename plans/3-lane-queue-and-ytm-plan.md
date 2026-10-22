# Master Architectural Plan: 3-Lane Queue System & YouTube Music Backend Integration

> **Document Version**: 1.0.0  
> **Status**: APPROVED ARCHITECTURE  
> **Target Subsystems**: `crates/rustic_core`, `crates/engine`, `crates/sources`, `crates/app`, `crates/ui-kit`  
> **Target Constraints**: 100% Pure Rust, Zero-C-DLLs, GPUI 60fps rendering, <50MB RAM RSS, ~0% idle CPU, <0.8% playback CPU.

---

## 1. Executive Summary & Core Architectural Invariants

RusticPlayer's queue architecture is structured as a **3-Tier Priority Queue with History and Automix Radio Pagination**.

```
┌────────────────────────────────────────────────────────────────────────┐
│ LANE 1: USER PRIORITY QUEUE (FIFO / Manual Priority)                  │
│ • Songs added via "Play Next" (push_front) or "Add to Queue" (push_back)│
│ • Strict priority over Context and Radio                              │
│ • Never shuffled; "Clear Queue" clears ONLY this lane                  │
├────────────────────────────────────────────────────────────────────────┤
│ LANE 2: CONTEXT QUEUE (Album / Playlist / Artist Collection)           │
│ • Tracks belonging to the active collection being played               │
│ • Maintained via `context_cursor`: tracks before are past, after next │
│ • Shuffle reorders only this lane; un-shuffle restores original order  │
├────────────────────────────────────────────────────────────────────────┤
│ LANE 3: AUTOMIX RADIO QUEUE (YouTube Music Endless Stream)             │
│ • Populated automatically via Innertube `/next` using `RDAMVM{videoId}`│
│ • Paginated via `continuation` tokens when count drops below 4         │
│ • Only plays when Lanes 1 & 2 are exhausted (or in single-song radio) │
│ • Toggling Autoplay OFF instantly hides/disables this lane             │
├────────────────────────────────────────────────────────────────────────┤
│ SESSION HISTORY (Capped at 50 Tracks)                                  │
│ • Records completed/skipped tracks for "Previous" navigation           │
│ • Auto-persisted to SQLite `history` table                             │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Mathematical State Machine & Advance Priority

Let $Q$ represent the state tuple:
$$Q = \langle \mathcal{U}, \mathcal{C}, i_c, \mathcal{R}, \mathcal{H}, A, S \rangle$$
Where:
- $\mathcal{U} \in \text{VecDeque}\langle\text{Track}\rangle$: User Priority Queue.
- $\mathcal{C} \in \text{Vec}\langle\text{Track}\rangle$: Context Queue (Album/Playlist).
- $i_c \in \mathbb{N}$: Context cursor index into $\mathcal{C}$.
- $\mathcal{R} \in \text{VecDeque}\langle\text{Track}\rangle$: Automix Radio Queue (Innertube `RDAMVM`).
- $\mathcal{H} \in \text{Vec}\langle\text{Track}\rangle$: Session History (bounded, $\le 50$).
- $A \in \{\text{true}, \text{false}\}$: Autoplay toggle.
- $S \in \{\text{true}, \text{false}\}$: Shuffle toggle.

### 2.1 Next Track Transition Function (`advance() -> Option<Track>`)

When the current track $T_{\text{cur}}$ ends or the user clicks "Next":
1. **Push History**: $\mathcal{H} \leftarrow \mathcal{H} \cup \{ T_{\text{cur}} \}$; if $|\mathcal{H}| > 50$, pop oldest.
2. **Evaluate Priority Ladder**:
   $$\text{NextTrack} = \begin{cases} 
   \text{pop\_front}(\mathcal{U}) & \text{if } \mathcal{U} \neq \emptyset \quad (\text{Lane 1}) \\
   \mathcal{C}[i_c + 1] \text{ with } i_c \leftarrow i_c + 1 & \text{else if } i_c + 1 < |\mathcal{C}| \quad (\text{Lane 2}) \\
   \text{pop\_front}(\mathcal{R}) & \text{else if } A = \text{true} \land \mathcal{R} \neq \emptyset \quad (\text{Lane 3}) \\
   \text{None} & \text{otherwise (Stop playback)}
   \end{cases}$$
3. **Threshold Check**: If $|\mathcal{R}| < 4 \land A = \text{true}$, dispatch background async task: `fetch_radio_continuation()`.
4. **Resolve-Ahead**: Immediately initiate pre-resolution of `NextTrack` + 1 audio stream URL and buffer first 64KB block.

### 2.2 Previous Track Function (`previous(pos_secs) -> PreviousAction`)
$$\text{Action} = \begin{cases}
\text{RestartTrack}(0.0) & \text{if } \text{pos\_secs} > 3.0 \\
\text{Play}(\text{pop}(\mathcal{H})) & \text{else if } \mathcal{H} \neq \emptyset \\
\text{RestartTrack}(0.0) & \text{otherwise}
\end{cases}$$

### 2.3 User Queue Operations
- **Play Next**: $\mathcal{U} \leftarrow \text{push\_front}(\mathcal{U}, T)$
- **Add to Queue**: $\mathcal{U} \leftarrow \text{push\_back}(\mathcal{U}, T)$
- **Clear User Queue**: $\mathcal{U} \leftarrow \emptyset$ *(Leaves $\mathcal{C}$ and current track completely untouched)*
- **Promote from Radio**: For index $k \in \mathcal{R}$, remove $T = \mathcal{R}[k]$, and $\mathcal{U} \leftarrow \text{push\_back}(\mathcal{U}, T)$.

---

## 3. YouTube Music Backend Integration (`crates/sources`)

### 3.1 Innertube Automix Radio (`RDAMVM`)
When playing any single track (from search, artist top tracks, or direct click), YouTube Music's `/youtubei/v1/next` endpoint returns an infinite algorithmic radio stream:

1. **Bootstrap Request**:
   - `POST https://music.youtube.com/youtubei/v1/next`
   - Payload:
     ```json
     {
       "context": { "client": { "clientName": "WEB_REMIX", "clientVersion": "1.20260213.01.00" } },
       "videoId": "kwH2nYwuFdA",
       "playlistId": "RDAMVMkwH2nYwuFdA",
       "isAudioOnly": true
     }
     ```
2. **Parser Updates in `crates/sources/src/ytmusic/parser.rs`**:
   - Extract initial 25 tracks from `playlistPanelRenderer.contents`.
   - Extract continuation token:
     `playlistPanelRenderer.continuations[0].nextContinuationData.continuation`.
   - Extract lyrics `browseId`: `MPLYt_...`.
3. **Continuation Fetcher**:
   - `POST https://music.youtube.com/youtubei/v1/next?continuation={token}`
   - Ingest next 25 items into $\mathcal{R}$ (`suggested_queue`).
   - Deduplicate incoming items against $\mathcal{U} \cup \mathcal{C} \cup \mathcal{H} \cup \mathcal{R}$.

### 3.2 Google History Registration Beacon
To ensure played tracks appear in official YouTube Music History and update "Listen Again":
1. In `PlayerResponse`, extract `playbackTracking.videostatsPlaybackUrl.baseUrl`.
2. Generate a 16-character alphanumeric Client Playback Nonce (`cpn`).
3. Make an HTTP GET request to `videostatsPlaybackUrl` with:
   - `ver=2`, `c=WEB_REMIX`, `cpn={cpn}`
   - `list={playlist_id}` (if playing a playlist/album)
   - Headers: `Authorization: SAPISIDHASH ...` + `Cookie: ...`

---

## 4. API Contract Changes (`crates/rustic_core/src/api.rs`)

### 4.1 New `CommandAction` Variants
```rust
pub enum CommandAction {
    // Existing playback controls...
    Play,
    Pause,
    TogglePlay,
    Stop,
    Next,
    Previous,
    Seek(Duration),
    SetVolume(f32),

    // --- 3-Lane Queue Commands ---
    /// Plays an album or playlist context, replacing Lane 2 (Context Queue).
    PlayContext {
        tracks: Vec<Track>,
        start_index: usize,
        context_id: String,
        context_title: String,
        shuffle: bool,
    },
    /// Plays a single track directly and triggers Lane 3 (RDAMVM Radio).
    PlaySingleTrack(Track),
    /// Enqueues a track to play immediately after the current song (Lane 1 Front).
    EnqueueNext(Track),
    /// Appends a track to the end of user queue (Lane 1 Back).
    EnqueueEnd(Track),
    /// Removes a track from Lane 1 (User Queue).
    RemoveUserQueueItem(usize),
    /// Clears only Lane 1 (User Queue), leaving context and current track intact.
    ClearUserQueue,
    /// Moves an item within Lane 1 (Drag-and-drop reorder).
    MoveUserQueueItem { from: usize, to: usize },
    /// Promotes an item from Lane 3 (Radio) into Lane 1 (User Queue).
    PromoteRadioItem(usize),
    /// Toggles Autoplay (enables/disables Lane 3).
    ToggleAutoplay(bool),
    /// Ingests more radio tracks from background continuation.
    IngestRadioTracks(Vec<Track>),
}
```

### 4.2 Multi-Lane Snapshot Event
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultiLaneQueueSnapshot {
    pub current_track: Option<Track>,
    pub user_queue: Vec<Track>,          // Lane 1
    pub context_title: Option<String>,
    pub context_upcoming: Vec<Track>,    // Lane 2
    pub radio_upcoming: Vec<Track>,      // Lane 3
    pub history: Vec<Track>,             // Past played
    pub autoplay_enabled: bool,
    pub is_shuffled: bool,
    pub generation: u64,
}

pub enum Event {
    // Existing events...
    PlaybackStateChanged(PlaybackState),
    PositionChanged { position_secs: f64, duration_secs: f64 },
    TrackChanged(Option<Track>),

    // --- Unified Multi-Lane Event ---
    QueueUpdated(Box<MultiLaneQueueSnapshot>),
}
```

---

## 5. Phased Implementation Roadmap

```
[Phase 1A: Core Data Models & API Contract]
   ├── Refactor QueueManager in crates/rustic_core/src/queue/mod.rs
   └── Add Commands and Events in crates/rustic_core/src/api.rs
           │
           ▼
[Phase 1B: Innertube Backend Enhancements]
   ├── Parse continuation token in crates/sources/src/ytmusic/parser.rs
   ├── Add fetch_radio_continuation() in crates/sources/src/ytmusic/client.rs
   └── Add Google History beacon trigger in crates/sources/src/ytmusic/stream.rs
           │
           ▼
[Phase 1C: EngineLoop Multi-Lane Adoption]
   ├── Replace naive state.queue in crates/engine/src/lib.rs with QueueManager
   ├── Wire advance(), play_next(), play_prev() to QueueManager
   └── Add background threshold prefetcher (Depth = 1 + Continuation)
           │
           ▼
[Phase 1D: Tokio ↔ GPUI Bridge & AppState Sync]
   ├── Update crates/app/src/engine_bridge.rs to consume MultiLaneQueueSnapshot
   └── Expose atomic command dispatchers for all 3 lanes
           │
           ▼
[Phase 1E: GPUI UI Perfection (QueuePanel & Up Next)]
   ├── Build 3-lane visual layout in crates/ui-kit/src/organisms/queue_panel.rs
   ├── Wire Now Playing, User Queue, Context, and Radio cards
   └── Add drag-to-reorder and click-to-promote interactions
```

---

## 6. Concrete File Targets & Mod Map

| Target File | Primary Responsibilities & Changes |
|---|---|
| [`crates/rustic_core/src/queue/mod.rs`](file:///f:/VSCodium/Github/RusticPlayer/crates/rustic_core/src/queue/mod.rs) | Formalize 3-lane structure (`user_queue: VecDeque`, `context_queue: Vec`, `suggested_queue: VecDeque`, `history: Vec`). Add advance priority, unshuffle preservation, and snapshot generator. |
| [`crates/rustic_core/src/api.rs`](file:///f:/VSCodium/Github/RusticPlayer/crates/rustic_core/src/api.rs) | Add 3-lane `CommandAction`s and `Event::QueueUpdated(Box<MultiLaneQueueSnapshot>)`. |
| [`crates/sources/src/ytmusic/parser.rs`](file:///f:/VSCodium/Github/RusticPlayer/crates/sources/src/ytmusic/parser.rs) | Parse `continuation` token and `automixPreviewVideoRenderer` from Innertube `/next` responses. |
| [`crates/sources/src/ytmusic/client.rs`](file:///f:/VSCodium/Github/RusticPlayer/crates/sources/src/ytmusic/client.rs) | Add `fetch_radio_continuation(token)` and `fetch_automix_radio(video_id)`. |
| [`crates/sources/src/ytmusic/stream.rs`](file:///f:/VSCodium/Github/RusticPlayer/crates/sources/src/ytmusic/stream.rs) | Implement `register_playback_beacon` (`videostatsPlaybackUrl` + CPN). |
| [`crates/engine/src/lib.rs`](file:///f:/VSCodium/Github/RusticPlayer/crates/engine/src/lib.rs) | Remove `state.queue: Vec<Track>`; embed `QueueManager`. Wire commands, events, and background resolve-ahead. |
| [`crates/app/src/engine_bridge.rs`](file:///f:/VSCodium/Github/RusticPlayer/crates/app/src/engine_bridge.rs) | Sync `AppState` with `MultiLaneQueueSnapshot`. Wire UI button actions to engine queue commands. |
| [`crates/app/src/pages.rs`](file:///f:/VSCodium/Github/RusticPlayer/crates/app/src/pages.rs) | Update `PlayerPage` and `QueueView` to render Lane 1, Lane 2, and Lane 3 with clean visual segregation. |
| [`crates/ui-kit/src/organisms/queue_panel.rs`](file:///f:/VSCodium/Github/RusticPlayer/crates/ui-kit/src/organisms/queue_panel.rs) | Pure GPUI presentation of the 3-lane cards with reordering and "Clear Queue" actions. |

---

## 7. Edge Cases, Risks & Mitigations

1. **Rapid Skipping ("Seek / Next Storm")**:
   - *Risk*: User spams Next button 10 times in 2 seconds, causing overlapping network calls and race conditions.
   - *Mitigation*: Engine actor loop runs linearly (`cmd_rx.recv()`). Each `Next` command synchronously updates `QueueManager` state immediately and cancels any pending resolve task before launching the new track's resolve.
2. **Network Drops on Continuation**:
   - *Risk*: Fetching continuation fails due to internet interruption.
   - *Mitigation*: Retry up to 3 times with exponential backoff (1s, 2s, 4s). If all fail, gracefully keep existing radio tracks and retry on next track advance without crashing.
3. **Radio Track Duplication**:
   - *Risk*: YouTube Music recommends songs already in the queue or recently played.
   - *Mitigation*: In `QueueManager::ingest_suggestions()`, enforce strict deduplication against $\mathcal{U} \cup \mathcal{C} \cup \mathcal{H} \cup \mathcal{R}$.
4. **Memory Constraint (<50MB RAM)**:
   - *Risk*: Long listening session (8+ hours) keeps hundreds of track structs in memory.
   - *Mitigation*: Bound `history` to max 50 tracks and `suggested_queue` to max 50 tracks. Completed tracks flush to SQLite database `history` table, keeping memory flat at ~250KB for the entire queue subsystem.

---

## 8. Explicit Non-Goals for Phase 1

- Multi-device sync / Spotify Connect equivalent (deferred to Jam Phase).
- Collaborative queue editing over LAN (deferred to Jam Phase).
- Full offline file synchronization (deferred to Phase 3; tracks will still stream from CDN if not cached).
