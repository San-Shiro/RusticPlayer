# Development Plan — Rust Music Player (GPUI)

**Stack:** Rust · GPUI + gpui-component · symphonia + cpal · SQLite
**Targets:** Linux (primary), Windows, macOS
**Strategy:** engine-first, headless and fully testable before any pixel is drawn
**Timeline:** ~20 weeks solo to v1.0 (adjust for your actual hours/week)

---

## Guiding principles

1. **The engine must be usable without the UI.** If you can't play an album from a CLI, the engine isn't done.
2. **`core` never imports GPUI.** Not once. This is what makes the frontend swappable and the plugin system possible.
3. **Everything crosses the boundary as events and commands.** No shared mutable state between engine and UI.
4. **Test the engine hard, test the UI lightly.** Audio bugs are invisible and infuriating; UI bugs are obvious.
5. **Plugins are not built in v1 — but the seams are cut in v1.** See Phase 9.

---

## Workspace layout

```
music-player/
├── crates/
│   ├── core/            pure logic. no I/O, no platform, no UI.
│   │                    playlist, queue, shuffle, repeat, state machines
│   ├── audio/           symphonia decode + cpal output + mixer
│   ├── library/         scanning, metadata, SQLite, search, artwork
│   ├── platform/        MPRIS / SMTC / MediaCenter, media keys, tray, paths
│   ├── engine/          orchestrates audio+library+core. THE public API.
│   ├── plugin-api/      (Phase 9) stable types shared with plugins
│   ├── cli/             headless client — your Phase 1-3 test harness
│   └── app/             GPUI frontend
└── Cargo.toml           workspace
```

**Dependency rule, enforced by review:**
`app → engine → {core, audio, library, platform}`
`core` depends on nothing but `serde` and `std`. `audio` never imports `library`. If you're tempted to break this, you're about to create the bug that makes the plugin system impossible later.

---

## The engine's public interface (design this on day one)

Everything the UI can do goes through two enums. Write these before you write any implementation.

```rust
// crates/engine/src/api.rs

pub enum Command {
    // transport
    Play, Pause, TogglePlay, Stop,
    Next, Previous,
    Seek(Duration),
    SetVolume(f32),

    // queue
    PlayTrack(TrackId),
    PlayContext { context: PlayContext, start_at: usize },
    EnqueueNext(Vec<TrackId>),
    EnqueueEnd(Vec<TrackId>),
    RemoveFromQueue(usize),
    MoveInQueue { from: usize, to: usize },
    SetShuffle(bool),
    SetRepeat(RepeatMode),

    // library
    ScanFolders(Vec<PathBuf>),
    RescanChanged,
    Search { query: String, limit: usize },
}

pub enum Event {
    // transport
    PlaybackStateChanged(PlaybackState),
    PositionChanged { position: Duration, duration: Duration },
    TrackChanged(Option<Track>),
    VolumeChanged(f32),

    // queue
    QueueChanged(Vec<Track>),
    ShuffleChanged(bool),
    RepeatChanged(RepeatMode),

    // library
    ScanProgress { done: usize, total: usize },
    ScanComplete { added: usize, updated: usize, removed: usize },
    LibraryChanged,
    SearchResults { query: String, results: SearchResults },

    // errors
    Error(EngineError),
}
```

`Engine::send(Command)` and `Engine::subscribe() -> Receiver<Event>`. That's the whole surface.

**Why this matters beyond tidiness:** the CLI, the GPUI app, and (later) every plugin are all just consumers of this same bus. Get it right once.

---

# PHASE 0 — Foundation
**Week 1**

### Build
- Cargo workspace, all crates stubbed
- CI: `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` on Linux + Windows + macOS runners
- Release profile: `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`
- `tracing` + `tracing-subscriber` wired everywhere. `tracy` feature flag behind `--features profiling`
- The `Command`/`Event` enums above, fully written, no implementation
- `thiserror` error types per crate

### Test
- CI green on all three platforms
- `cargo test` runs (even with zero tests)
- A dependency-direction test: a script or `cargo-deny` config that fails if `core` gains any dependency

### Done when
Empty binaries build and run on all three OSes from a clean checkout.

---

# PHASE 1 — Audio Engine
**Weeks 2–4 · the hardest and most important phase**

### 1a. Decode pipeline (week 2)

**Build**
- `symphonia` integration: FLAC, MP3, AAC/M4A, OGG/Vorbis, Opus, WAV
- Probe format → get `duration`, `sample_rate`, `channels`, codec
- Decode to `f32` interleaved packets
- `rubato` resampler when file rate ≠ device rate
- Error handling for corrupt/truncated files — must never panic

**Test**
- Assemble a **test corpus**: one file per codec, plus deliberately broken ones (truncated FLAC, zero-byte MP3, wrong extension, 0-length file, file with no audio stream)
- Unit test: decode each → assert sample count matches duration × rate × channels (±1 frame)
- Unit test: every broken file returns `Err`, never panics
- Property test with `proptest`: random byte garbage into the probe never panics

### 1b. Output + ring buffer (week 3)

**Build**
- `cpal` device enumeration, default device, stream setup
- **Lock-free ring buffer** (`rtrb`) between decoder thread and audio callback
- Audio callback: reads ring buffer only. **No allocation, no locks, no logging, no `println!`.** This is a hard rule.
- Decoder thread fills ahead; target ~500ms of buffer
- Volume as an atomic `f32`, applied in the callback
- Underrun detection → count and report, output silence, never glitch-loop

**Test**
- Manual: play a file end to end, no glitches, for 10 minutes
- Automated: mock output sink consuming at real-time rate — assert zero underruns over 60s of simulated playback
- Assert callback allocation-free: run under a custom global allocator that panics if called from the audio thread (do this, it catches real bugs)
- Test device disconnect mid-playback (unplug USB DAC / switch default device) — must recover or error cleanly, not hang

### 1c. Transport + state machine (week 4)

**Build**
- `PlaybackState`: `Stopped | Playing | Paused | Buffering`
- Play, pause, stop, seek (seek = flush ring buffer + reposition decoder)
- Accurate position reporting — derive from samples consumed by the callback, **not** wall clock
- **Gapless playback**: pre-decode the next track's opening while the current one finishes
- Volume with a smoothing ramp (avoid clicks)
- Optional: ReplayGain from tags, crossfade

**Test**
- State machine unit tests: every transition, including invalid ones (`Pause` while `Stopped` is a no-op, not an error)
- Seek accuracy: seek to 30s in a known file, assert reported position within 50ms
- Seek storm: 100 random seeks in 10s — no crash, no drift, no leak
- Gapless: play two files that form a continuous tone, record output, assert no gap or click at the boundary
- Position monotonicity: position never goes backwards during normal playback

### Phase 1 done when
`cargo run -p cli -- play album/` plays a full gapless album with working pause/seek/volume, and survives a seek storm and a device switch.

---

# PHASE 2 — Library & Metadata
**Weeks 5–6**

### Build
- **Scanner**: `jwalk` parallel directory walk, filter by extension
- **Metadata**: `lofty` for tags — title, artist, album, album artist, track/disc no, year, genre, duration, embedded artwork
- **SQLite** via `rusqlite` (bundled feature, so no system dep):
  - `tracks`, `albums`, `artists`, `playlists`, `playlist_tracks`
  - Store `mtime` + `size` per file for cheap incremental rescan
  - Indices on album_id, artist_id, path
- **FTS5** virtual table for search over title/artist/album
- **Artwork**: extract embedded art → generate 64px, 256px, and 640px thumbnails → cache to disk as WebP, keyed by content hash
- Incremental rescan: compare mtime/size, only re-read changed files
- Handle: missing tags (fall back to filename), multi-disc albums, compilations (album artist ≠ artist), various-artists

### Test
- **Fixture library** of ~50 files covering: full tags, no tags, unicode/CJK/emoji in tags, multi-disc, compilation, duplicate titles, very long strings, embedded art of several sizes, no art
- Scan fixture → assert exact track/album/artist counts
- Rescan unchanged → assert **zero** file reads (instrument it)
- Modify one file's tags → rescan → assert only that row updates
- Delete a file → rescan → assert removal
- Search tests: partial match, unicode, case-insensitive, accent-insensitive
- **Benchmark**: scan 10,000 files. Target under 60s cold on an SSD. If it's much worse, fix it now — it only gets worse.
- Thumbnail cache: assert cache hit on second request, assert content-hash dedupe works

### Phase 2 done when
`cargo run -p cli -- scan ~/Music` handles 10k files in reasonable time, `cli search "query"` returns sane results instantly, and rescan of an unchanged library is near-instant.

---

# PHASE 3 — Engine Integration + Headless CLI
**Week 7 · the phase people skip and regret**

### Build
- `engine` crate wires audio + library + core behind `Command`/`Event`
- Queue management: shuffle (Fisher-Yates with a stored seed so it's reproducible), repeat off/all/one, play-next vs enqueue
- Auto-advance on track end
- Persist state across restart: queue, position, volume, shuffle/repeat
- **CLI client** — a real one:
  ```
  cli scan <path>
  cli search <query>
  cli play <track-id | path | album>
  cli queue [list|add|next|clear]
  cli status          # prints live events as they stream
  cli seek <seconds>
  ```

### Test
- **Integration tests against the engine API only** — these are your most valuable tests and they'll outlive every UI decision
- Scripted scenario: scan → search → play → next → seek → pause → resume → verify event sequence
- Shuffle: same seed → same order; all tracks appear exactly once
- Repeat-one, repeat-all, repeat-off boundary behaviour at queue end
- Restart persistence: kill process mid-playback, restart, assert queue and position restored
- **Soak test: play a 200-track queue end to end unattended.** Assert no crash, and RSS at the end within 10% of RSS after track 5. This catches leaks nothing else will.

### Phase 3 done when
You can use the CLI as your daily music player for a day. Seriously do this — you'll find five bugs.

---

# PHASE 4 — GPUI Shell
**Weeks 8–10 · first pixels**

### Build
- `app` crate: `gpui` + `gpui_platform` + `gpui-component`, pinned to specific git SHAs
- Window, root view, `gpui_component::init`
- Theme: pick a base theme, define your palette and type scale
- Lucide SVG icon set added to assets
- **Layout skeleton**: sidebar (nav) · main content · now-playing bar
- **Engine bridge**: spawn engine on a background executor; forward `Event`s onto the foreground executor to update view state; views dispatch `Command`s. One module, `engine_bridge.rs`, nothing else touches the engine.
- Responsive: sidebar collapses to icons below ~900px; grid columns 5→3→2

### Test
- Manual on **Wayland and X11** — both, from the start
- Fractional scaling at 100/125/150/200% — this is where Linux apps visibly break
- Windows + macOS smoke test
- **Idle check: window open, nothing playing → assert ~0% CPU.** If not, find the repaint loop now.
- Resize from 600px to 4K, no layout breakage
- Startup timing: instrument first-pixel. Target <150ms.

### Phase 4 done when
The shell renders, resizes correctly on both display servers, idles at 0%, and a hardcoded button can start playback through the real engine.

---

# PHASE 5 — Core UI Features
**Weeks 11–14**

### Build
- **Library views**: albums grid, artists list, all-tracks table — all on `gpui-component`'s virtualized List/Table
- **Album detail**: header with art, track list, play/shuffle actions
- **Now-playing bar**: art, title/artist, transport controls, seek bar, volume
- **Queue panel**: current queue, reorder, remove
- **Search**: instant, debounced ~150ms, grouped results (tracks/albums/artists)
- **Playlists**: create, rename, delete, add/remove tracks
- **Image pipeline**: request thumbnail at display size, decode off-thread, LRU cache bounded to ~20 full-size + ~200 thumbs, never decode on the UI thread
- Keyboard shortcuts: space, arrows, `/` for search, ctrl+F

### Test
- **10k-track library scroll test**: hold scroll for 30s. Frame time p99 under 8ms. Memory flat.
- Memory ceiling: navigate 50 albums with art → RSS stays under 150MB
- Search responsiveness: keystroke to results under 100ms on 10k tracks
- Rapid interaction: click 20 tracks in 10s, no crash, no stuck state
- Empty states: no library, no results, no artwork, empty playlist
- Long strings: 200-char track titles, CJK, RTL text — no layout explosion

### Phase 5 done when
You can replace your current music player with it for everyday use.

---

# PHASE 6 — Polish & Effects
**Weeks 15–17**

### Build
- **Backdrop blur** from album art (GPUI provides it — cache the result, invalidate only on track change)
- Transitions: view changes, hover states, now-playing art crossfade
- Drag-and-drop playlist reorder — **scoped**: within one list, no ghost preview, insert-line indicator only. Add autoscroll after.
- Context menus (right-click track/album/playlist)
- Marquee for overflowing titles
- Loading skeletons
- Light/dark theme + system preference detection
- Settings: library folders, audio device, theme, behaviour

### Test
- **Blur cost**: measure frame time with blur on vs off. Delta must be near zero when static (proves the cache works).
- Animation smoothness: `tracy` capture during transitions, p99 under 8ms
- Drag-drop edge cases: drop on self, drop outside list, escape mid-drag, drag while filtered
- Theme switch mid-playback, no flicker
- **Re-run the idle test** — polish is where idle CPU regressions creep in

### Phase 6 done when
It looks like something you'd screenshot, and idle is still ~0%.

---

# PHASE 7 — Platform Integration
**Week 18**

### Build
Behind one trait in `platform/`:
- **Linux**: MPRIS via `zbus` — metadata, transport, position, seek. This is what makes it feel native on Linux.
- **Windows**: SMTC via `windows-rs`
- **macOS**: `MPNowPlayingInfoCenter`
- Media keys on all three
- Tray icon (optional, minimize-to-tray)
- Single-instance enforcement
- Prevent sleep during playback

### Test
- Linux: `playerctl play-pause` works; GNOME/KDE media widget shows correct art and position
- Windows: media overlay appears on volume keys
- macOS: Control Center / Now Playing shows track
- Bluetooth headphone play/pause button works on each platform
- Second instance launch focuses the existing window rather than opening a new one

---

# PHASE 8 — Packaging & Release
**Weeks 19–20**

### Build
- Linux: AppImage + Flatpak (Flatpak matters most for distribution), `.desktop` file, icons
- Windows: MSI or NSIS installer, code signing
- macOS: `.app` bundle, notarization
- Auto-update check (optional)
- Crash reporting with opt-in
- First-run experience: pick library folders, scan with progress

### Test
- Install from package on clean VMs: Ubuntu, Fedora, Arch, Windows 11, macOS
- Verify no missing runtime deps (this is the classic Linux packaging failure)
- Cold-start timing on the packaged build
- Uninstall leaves no orphaned files
- **Full soak: 8-hour continuous playback.** Memory flat, no crash.

---

# PHASE 9 — Jam / Party Play
**Weeks 21–28 · v1.1 feature, but seams cut in Phase 0**

### Transport is pluggable — do not hardcode one

The session logic must not know how bytes move. Define this early:

```rust
#[async_trait]
pub trait JamTransport: Send + Sync {
    async fn advertise(&self, session: &SessionInfo) -> Result<()>;
    async fn discover(&self) -> Result<Vec<SessionInfo>>;
    async fn join(&self, code: &str) -> Result<PeerChannel>;
    async fn send(&self, to: PeerId, msg: &WireMessage) -> Result<()>;
    fn incoming(&self) -> Receiver<(PeerId, WireMessage)>;

    fn capabilities(&self) -> TransportCaps; // latency class, bandwidth, audio-stream capable
}
```

Planned implementations:

| Impl | Reach | Notes |
|---|---|---|
| **LAN** | Same network | mDNS (`_musicjam._tcp.local`) + WebSocket/QUIC. Low latency, viable for Mode B sync. |
| **Cloud relay** (your approach) | Internet | Code-pairing model. Higher and more variable latency — treat as Mode A capable, Mode B only if measured RTT allows. |
| **Direct P2P** | Internet | WebRTC data channels + STUN/TURN. Fallback if the relay path becomes unavailable. |

`TransportCaps` is what makes this work: the session layer queries latency class and decides whether synchronized playback (Mode B) is even offered, or whether it degrades to collaborative queue (Mode A). Don't let the UI promise sync the transport can't deliver.

**Rule:** ship at least two transports before v1.1. A single undocumented dependency that can change without notice is not a foundation — but as *one* implementation behind a trait, it's a good one.

### The architectural gift you already have

**A Jam peer is the same thing as a plugin: another producer of `Command`s and consumer of `Event`s.** You already decided the bus would be `Serialize`/`Deserialize` in Phase 0. That means the hard part of Jam isn't the session logic — it's clock sync and transport. The control plane is nearly free.

```
Guest UI → Command → [network] → Host engine → Event → [network] → all peers → Guest UI
```

### 9a. Decide the mode (do this before writing code)

These are very different products. Pick one for v1.

**Mode A — Collaborative Queue** (easy, ~2 weeks)
One host device plays audio out loud. Guests join and add/vote/reorder the queue from their own devices. No audio sync at all.
*This is a party-speaker feature. It solves 80% of the real use case for 20% of the work.*

**Mode B — Synchronized Playback** (hard, ~6 weeks)
Every device plays the same audio at the same instant through its own output. Requires clock sync and drift correction.
*This is the Spotify Jam experience. It's also where all the difficulty lives.*

**Recommendation: ship Mode A first.** It's genuinely useful, it validates discovery/transport/session/permissions, and Mode B is then purely an audio problem layered on top of working infrastructure.

### 9b. Session layer (week 21–22)

**Build**
- Host creates a session → generates a **6-character join code** and an **ephemeral keypair**
- Advertise via whichever `JamTransport` is active — LAN uses mDNS TXT records, cloud relay uses code registration
- Guest discovery: enumerate across all enabled transports, or enter a code manually
- **End-to-end encrypt the payload regardless of transport.** If the relay is third-party, it should carry ciphertext only — this also means a transport swap doesn't change your security story.
- WebSocket control channel, **host-authoritative** (do *not* reach for CRDTs — there is always a host, and last-write-wins with a single authority is dramatically simpler)
- Roles: `Host` (full control) · `Guest` (add to queue, vote) · `Listener` (read-only)
- Host can promote, kick, and lock the queue
- Handshake: code → ECDH → session key → all traffic encrypted (`snow` for Noise protocol, or just TLS)

**Test**
- Two processes on one machine, then two machines on one LAN
- Guest joins, adds a track, host sees it within 200ms
- Wrong code rejected; rate-limit join attempts (brute-forcing 6 chars is trivial otherwise)
- Host disconnects → guests get a clean "session ended", no hang
- Guest disconnects mid-action → host state stays consistent
- 8 guests simultaneously adding tracks → no lost updates, no duplicate ordering

### 9c. Shared queue semantics (week 23)

**Build**
- Guest `Command`s are *requests*; host validates and applies, then broadcasts the resulting `Event`
- Per-guest rate limit (e.g. 3 pending additions each) so one person can't flood the queue
- Attribution: every queued track carries who added it
- Optional: upvoting, with the queue sorted by votes above a pinned "playing next"
- Conflict rule: host's local action always wins

**Test**
- Concurrent add from 4 guests → final order is deterministic and matches host's view on every device
- Rate limit enforced
- Guest tries a host-only command → rejected, doesn't crash
- Queue state converges after a guest reconnects (send full snapshot on reconnect, not a diff)

### 9d. Clock synchronization (weeks 24–25) — Mode B only

This is the genuinely hard part. Read Snapcast's implementation before writing a line; it is the open-source prior art for exactly this problem.

**Build**
- SNTP-style offset estimation: guest sends `t1`, host replies with `t2`/`t3`, guest records `t4`. Offset ≈ `((t2-t1) + (t3-t4))/2`, RTT = `(t4-t1) - (t3-t2)`
- Keep a rolling window of ~20 samples; **use the minimum-RTT sample**, not the mean — it's the least jitter-contaminated
- Re-sync every ~5s, smooth the offset (don't jump)
- Playback scheduling: host broadcasts "play track X, sample 0 at shared-clock time T". Each guest buffers and starts at its local equivalent of T.
- **Drift correction by resampling, never by dropping samples.** Adjust playback rate by ±0.05–0.1% via `rubato` to converge. Dropping or duplicating samples is audible as clicks.

**Targets**
- Different rooms: <30ms drift is imperceptible
- Same room, multiple speakers: **<5ms**, or you get comb filtering and it sounds broken
- Buffer 300–1000ms on guests to absorb jitter (higher = more robust, more latency before playback starts)

**Test**
- Two devices, same track, physically adjacent — measure with a mic recording both, assert offset under 5ms
- Sustained sync: 30-minute playback, drift stays bounded (this catches clock-rate skew, which offset correction alone won't fix)
- Introduce artificial network jitter (`tc netem` on Linux) at 50ms/100ms/200ms — playback stays smooth
- Guest joins mid-track → syncs to the correct position within 2s
- WiFi handoff / brief disconnect → recovers without restarting the track

### 9e. Audio delivery (week 26) — Mode B only

Two strategies; support both.

**Strategy 1 — Same file, local playback.** If the guest already has the track (shared library, both own the file), send only "play track hash H at time T". Zero bandwidth, perfect quality. Match by audio content hash, not path or tags.

**Strategy 2 — Host streams.** Host encodes to Opus (~128kbps, 20ms frames) and sends over UDP/RTP or QUIC datagrams. Works for anything, costs bandwidth, needs FEC or a small retransmit window for packet loss.

**Test**
- Content-hash matching finds the same track across two differently-tagged libraries
- Opus path: assert quality and that 5% simulated packet loss doesn't produce audible dropouts
- Bandwidth ceiling with 8 guests
- Fallback: guest lacking the file automatically switches to streaming

### 9f. Jam UI (weeks 27–28)

**Build**
- "Start a Jam" entry point → session code + QR code
- Join flow: nearby sessions list (from mDNS) + manual code entry
- Participant list with avatars/names and roles
- Queue items show who added them
- Host controls: lock queue, kick, end session
- Connection status indicator; degraded/reconnecting states
- Sync health indicator for Mode B (drift in ms, for debugging — hide it behind a dev flag in release)

**Test**
- Full flow on three real devices
- Every failure state has a UI: host left, network dropped, kicked, session full, version mismatch
- Joining a session running a different app version → clean rejection with a readable message

### What to do in Phase 0 to make this cheap later

Add these to the Phase 0 checklist:

1. **`Command`/`Event` already `Serialize`/`Deserialize`** — already planned, now doubly justified
2. **Add `PeerId` and `origin: Option<PeerId>` to `Command`.** Costs nothing now; retrofitting attribution later touches every call site.
3. **Add `const PROTOCOL_VERSION`** alongside `API_VERSION`, and reject mismatched peers explicitly
4. **Content-hash every track at scan time** (Phase 2) — store `blake3` of the decoded audio, not the file bytes, so re-tagged copies still match. This is the single cheapest thing you can do now that makes Mode B far better later.
5. **Keep the audio clock derived from samples-consumed** (already specified in Phase 1c). Wall-clock position makes sync impossible.

### Risk notes

| Risk | Reality |
|---|---|
| Mode B on WiFi | Consumer WiFi jitter is the enemy, not bandwidth. Buffer generously. |
| Bluetooth output | BT adds 100–300ms of variable, undiscoverable latency. Offer a manual per-device offset slider — you cannot solve this automatically. |
| Same-room sync | 5ms is a genuinely tight target. Test with a microphone, not by ear. |
| Internet mode | Needs infrastructure you'd have to run and pay for. Defer past v1.1. |
| Abuse | A 6-char code is 2 billion combinations but people will share codes. Rate-limit joins and give hosts a kick button from day one. |

### Phase 9 done when
Mode A: three people at a party can add songs from their phones/laptops and it just works.
Mode B: two devices in the same room play the same track with no audible echo.

---

# PHASE 10 — Plugins (future, but seams cut now)

Addons are hard — you're right. Here's what to actually think about.

### The decision: WASM, not dynamic libraries

Rust has no stable ABI. Loading `.so`/`.dll` plugins compiled by other people means either a C ABI (painful, unsafe) or `abi_stable` (workable but restrictive), and *every plugin breaks when you bump your compiler*. Don't.

**Use WASM via `extism` or `wasmtime` + the Component Model.** You get:
- Sandboxing — a bad plugin can't corrupt your process or read arbitrary files
- Cross-platform — one plugin binary works everywhere
- ABI stability across your compiler versions
- Plugins written in Rust, Go, Zig, AssemblyScript, whatever
- The cost: a serialization boundary, and no real-time audio DSP (WASM in the audio callback is a bad idea)

For lighter customization — themes, small scripts — **Lua via `mlua`** is a good second tier and far easier for users.

### What plugins should be allowed to do (v1 of the plugin system)

| Capability | Verdict |
|---|---|
| Subscribe to `Event`s | ✅ Yes — this is the core value |
| Send `Command`s | ✅ Yes, rate-limited |
| Read library metadata | ✅ Yes |
| Provide metadata (scrobbling, lyrics, artist bios) | ✅ Yes — the obvious first use case |
| Provide a content source (streaming service, podcast feed) | ✅ Yes, via a `MediaSource` trait |
| Add a settings panel | ⚠️ Declarative schema only — you render it |
| Custom UI views | ❌ Not in v1. Foreign render code in GPUI is genuinely very hard. |
| Audio DSP in the callback | ❌ Not via WASM. Native-only, if ever. |

That table is the honest scope. "Plugins that add UI" is what makes plugin systems take a year; "plugins that observe events and provide metadata" is a few weeks and covers Last.fm scrobbling, lyrics, Discord presence, and custom sources — which is 90% of what people actually want.

### What to do NOW (costs you almost nothing)

1. **Keep `Command`/`Event` serializable.** `#[derive(Serialize, Deserialize)]` on every variant, from Phase 0. This is the single highest-leverage thing.
2. **Make the event bus multi-consumer.** Broadcast, not single-receiver. The UI is consumer #1; plugins are #2..N.
3. **Version the API.** Add a `const API_VERSION` and think about compatibility before you have users.
4. **Never let the UI reach into engine internals.** Every shortcut you take here is a wall you build against plugins later.
5. **Define `MediaSource` as a trait now**, even with only one implementation (local files). If local playback goes through the same trait a plugin would use, the abstraction is proven.

Do those five things and Phase 10 is a feature. Skip them and it's a rewrite.

**Note the overlap with Jam:** a remote peer and a plugin are the same abstraction — an external actor sending `Command`s and receiving `Event`s. Build Phase 9's peer authorization and rate-limiting with that in mind and Phase 10 inherits most of it.

---

## Testing summary by layer

| Layer | Type | When |
|---|---|---|
| `core` | Unit + property tests, high coverage | Continuous |
| `audio` | Unit + allocation assertions + manual listening | Phase 1 |
| `library` | Fixture-based integration + benchmarks | Phase 2 |
| `engine` | **Scenario integration tests — your crown jewels** | Phase 3, forever |
| `app` | Manual + performance profiling. Don't chase UI unit tests. | Phase 4+ |
| End-to-end | Soak tests, packaged-build installs | Phases 3, 5, 8 |

**Run every phase, every time:** idle CPU check, memory-flat soak, p99 frame time.

---

## Risk register

| Risk | Mitigation |
|---|---|
| GPUI breaking changes | Pin git SHAs. Bump deliberately, one at a time, never mid-feature. |
| gpui-component and gpui SHAs drift apart | Test the pair together before committing a bump. |
| Audio glitches on some hardware | Test on at least 3 devices incl. USB DAC and Bluetooth. Configurable buffer size. |
| Wayland/X11 divergence | Test both every phase, not at the end. |
| Memory creep from artwork | Bounded LRU from day one, soak test every phase. |
| Scope creep into plugins early | Phase 9 is Phase 9. Ship v1 first. |

---

## The first week, concretely

1. Create the workspace and all crate stubs
2. Write `Command` and `Event` in full, with `Serialize`/`Deserialize`, plus `PeerId` and `origin: Option<PeerId>` on `Command`
3. Declare `API_VERSION` and `PROTOCOL_VERSION`
4. Get CI green on three platforms
5. Assemble the audio test corpus (including the broken files)
6. Decode one FLAC to `f32` samples and assert the sample count

Items 2 and 3 are five minutes of work each and are what make Jam and plugins tractable in a year. Everything else on this list is throwaway scaffolding by comparison.

Nothing about GPUI for seven weeks. That's the plan working as intended.
