# Task Board: Innertube & Termusic Migration

## Active Worker Roster

| Worker ID | Engine / Model | Assignment Mode | Assigned Tasks |
| :--- | :--- | :--- | :--- |
| **Worker 0 (Host)** | Antigravity Flash | Inline Synchronous | T1.1, T1.2, T1.3, T2.2, T2.3, T3.1, T3.2, T4.1, T4.2, T4.3, T5.1, T5.2 |
| **Worker 1 (Operator/Oracle)** | OpenCode Claude Opus 4.8 | Background Dispatched (`oc_dispatch`) | T1.4 (Stream Resolver - Job `5bd55be8` ✅), T2.1 (Range-Buffered MediaSource - Next) |
| **Worker 2 (Oracle)** | Direct Postman MCP Claude Opus 4.8 | Inline Invariant Check (`oracle` / `context`) | Audio seek invariants, Content-Range protocol audit |

---

## Detailed Task Checklist

### Phase 1: Authenticated Innertube Client & Parser (`crates/sources`)

| Task ID | Description | Assigned Worker | Target Files | Status | Output / Verification |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **T1.1** | Enhance Session & Cookie Auth with multi-hash `SAPISIDHASH` (`origin: https://music.youtube.com`), `cookies.json` loader, and visitorData fallback | **Worker 0 (Host)** | `crates/sources/src/ytmusic/auth.rs`<br>`crates/sources/src/ytmusic/types.rs` | ✅ Completed | 2 passed unit tests; authenticates live sessions cleanly |
| **T1.2** | Build Resilient Polymorphic Innertube JSON Parser (handles all shelf variants, runs vs. simpleText, continuation tokens, lyrics browseId) | **Worker 0 (Host)** | `crates/sources/src/ytmusic/parser.rs` | ✅ Completed | Fully tested; parses shelves, carousels, queue, and lyrics |
| **T1.3** | Wire Client & verify live YouTube Music response | **Worker 0 (Host)** | `crates/sources/src/ytmusic/client.rs` | ✅ Completed | Live verification passed: parsed 2 real sections with 10 items each from YouTube Music |
| **T1.4** | Innertube Audio Stream Resolver (`/youtubei/v1/player` via `VISIONOS` context, itag 140 AAC preference, expire timestamp) | **Worker 1 (Oracle)** | `crates/sources/src/ytmusic/stream.rs` | ✅ Completed | Dispatched (Job `5bd55be8`), committed, 10 unit tests passing; 15/15 total in `crates/sources` |

---

### Phase 2: Termusic Playback Engine & Range HTTP Source (`crates/audio` & `crates/engine`)

| Task ID | Description | Assigned Worker | Target Files | Status | Output / Verification |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **T2.1** | Implement Range-Buffered Seekable `MediaSource` for Symphonia (HTTP range reader with persistent connection pool, instant Content-Range length discovery, sliding chunk cache, backward seek for MP4 `moov` atom) | **Worker 1 (Operator/Oracle)** | `crates/audio/src/http_source.rs` | ✅ Completed | Pure-Rust HTTP range reader with async prefetcher, backward seek, chunk cache; passes unit tests |
| **T2.2** | Wire `HttpSource` into `crates/audio::DecoderImpl` & verify with audio probe | **Worker 0 (Host)** | `crates/audio/src/decoder.rs`<br>`crates/audio/src/source.rs` | ✅ Completed | Symphonia probe recognizes remote stream headers; 22,400 samples decoded across 10 blocks in `test_playback.rs` |
| **T2.3** | Connect `crates/sources` stream resolver to `crates/engine::EngineLoop` for real YouTube audio playback | **Worker 0 (Host)** | `crates/engine/src/lib.rs`<br>`crates/engine/src/resolver.rs` | ✅ Completed | `TrackSource::YouTubeMusic` resolves stream URL via `VISIONOS` and streams AAC cleanly |

---

### Phase 3: Reactive Tokio ↔ GPUI Bridge (`crates/app`)

| Task ID | Description | Assigned Worker | Target Files | Status | Output / Verification |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **T3.1** | Implement runtime-agnostic bridge between Tokio and GPUI with 4Hz progress drop-oldest throttling | **Worker 0 (Host)** | `crates/app/src/engine_bridge.rs` | ✅ Completed | Coarse 250ms (4Hz) ticker resyncs DAC position and eliminates 60Hz whole-window reflow |
| **T3.2** | Reconcile 60fps seekbar interpolation with authoritative 4Hz ticks; seekbar scrub freeze & resume | **Worker 0 (Host)** | `crates/app/src/engine_bridge.rs`<br>`crates/ui-kit/src/atoms/seek_bar.rs`<br>`crates/ui-kit/src/atoms/slider.rs` | ✅ Completed | Analytical `PlaybackTimeline` with exponential decay (`CORRECTION_TAU: 0.100`), scrub freeze, and smooth thumb glide |

---

### Phase 4: Dynamic Frontend UI Population (`crates/app/src/pages.rs`)

| Task ID | Description | Assigned Worker | Target Files | Status | Output / Verification |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **T4.1** | Replace dummy home cards with live `state.home_feed.sections` carousels and mood filter chips | **Worker 0 (Host)** | `crates/app/src/pages.rs` | ✅ Completed | Dynamically renders live carousels ("Trending community playlists", "New releases") with functional mood filter chips |
| **T4.2** | Connect UP NEXT queue to `state.queue_snapshot` (50 tracks with RDAMVM) and live lyrics to `state.lyrics_doc` | **Worker 0 (Host)** | `crates/app/src/pages.rs` | ✅ Completed | 50 real YouTube Music radio tracks rendered with click-to-play; live synchronized line-by-line lyrics with timing sync |
| **T4.3** | Dynamic Gaussian ambient radiance adapting to current track artwork | **Worker 0 (Host)** | `crates/app/src/main.rs`<br>`crates/app/src/engine_bridge.rs` | ✅ Completed | Dynamic ambient radiance tint (`ambient_c1`, `ambient_c2`) overlaid on backdrop adapting to track palette |

---

### Phase 5: Verification & Hardening

| Task ID | Description | Assigned Worker | Target Files | Status | Output / Verification |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **T5.1** | CPAL WASAPI sleep/wake and device disconnect recovery state machine | **Worker 0 (Host)** | `crates/audio/src/output.rs`<br>`crates/audio/src/player.rs` | ✅ Completed | Auto-reconnect & immediate reconnect on Play; `MasterClock::on_reconnect` anchors playback continuity; 20/20 tests passing in `crates/audio` |
| **T5.2** | End-to-end integration test & visual screenshot verification | **Worker 0 (Host)** | `crates/app` | ✅ Completed | Captured and visually verified `ytm_live_home_verified.png`, `ytm_live_queue_verified.png`, and `ytm_live_lyrics_verified.png` with 8MB stack |
