# Rustic Player — Master Plan (Best-of-4 Synthesis, All-Rust)

**Goal:** A streaming-first, local-capable music player in 100% Rust — engine, backend, frontend — fast, <150MB RAM, ~0% idle CPU. Combines the best of Spotube, SimpMusic, ViVi Music, Namida. Spotify Jam-style multi-device play with pluggable transports and no mandatory central server.

**Confirmed decisions (from user):**
- Catalog: **YTM (Innertube) + Spotify both**, behind a `SourceProvider` trait, from the start
- Jam: **P2P-first** with pluggable transport layer (loopback for testing, LAN, WebRTC, cast-receiver-style fallback)
- Local files: **full multi-source from start** (Namida-style unified library)
- Engine-first ordering: playable engine → YT auth + library population → UI → services → Jam

**Reference docs:** `plans/development-plan.md` (engine/Jam/plugin deep detail — still authoritative for phases it covers), `plans/ui-framework-decision.md` (GPUI + gpui-component, performance rules), `reports/*.md` (the 4 source-app analyses).

---

## 1. Best features to steal, mapped to Rust

| Source app | Feature | Our Rust implementation |
|---|---|---|
| **Spotube** | SourcedTrack resolution flow + `sourceMatchTable` SQLite cache | `crates/sources/src/resolver.rs` + `source_matches` table |
| **Spotube** | `rankResults()` match scoring (artist match, `official\s(video|audio)` regex, duration delta) | `crates/core/src/matching.rs` — pure, property-tested. **Upgrade: match by ISRC first** (Spotify provides it), fall back to title/artist scoring |
| **Spotube** | Connect (mDNS + WebSocket remote control) | Evolves into `crates/jam` LAN transport |
| **Spotube** | SponsorBlock auto-skip | `crates/services/src/sponsorblock.rs` — REST poll per resolved video ID, position monitor in engine |
| **SimpMusic** | Innertube YTM scraper (auth, browse, radio, stream URLs) | `crates/sources/src/ytmusic/` — pure-Rust Innertube client (JSON POSTs to `youtubei/v1`), no yt-dlp dependency for v1 |
| **SimpMusic** | `CrossfadeExoPlayerAdapter` dual-player crossfade | `crates/audio/src/crossfade.rs` — two symphonia decode pipelines feeding one mixer, equal-power ramp |
| **SimpMusic** | `-empty` stub modules (FOSS/full flavors) | Cargo **feature flags**: `lastfm`, `discord`, `ai` — off by default |
| **ViVi** | SimpleCache-style offline chunk cache for streams | `crates/audio/src/stream_cache.rs` — HTTP range chunk cache on disk, keyed by video ID + itag, replay without re-download |
| **ViVi** | Word-by-word karaoke lyrics | Lyrics data model stores `Vec<Word { text, start, end }>`; providers that only give lines fill line-level timings |
| **ViVi** | ReplayGain / EQ / pitch-tempo | ReplayGain in Phase 1 (cheap), parametric EQ + tempo in Phase 8 (DSP behind the mixer) |
| **Namida** | Unified multi-source `Track` model | `core::Track { source: TrackSource, .. }` — one model for local/YTM/Spotify |
| **Namida** | Waveform extraction for seekbar | `crates/library/src/waveform.rs` — decode peaks off-thread, cache normalized `[u8; N]` per track |
| **Namida** | Native tag read/write | `lofty` (pure Rust — no TagLib FFI needed) |
| **All four** | Dynamic palette from album art | Phase 8: extract dominant colors off-thread, tint now-playing |
| **Deferred seams** | JioSaavn, Subsonic/WebDAV/SMB (Namida), AI playlists (SimpMusic), Canvas videos | `SourceProvider`/`MediaSource` traits make these drop-in later; do NOT build now |

---

## 2. Workspace layout (extend existing)

Current: `core, audio, library, platform, engine, plugin-api, cli, app` — keep, add three:

```
crates/
├── core/        pure logic. models, queue/shuffle/repeat, matching.rs (YouTube match scoring)
├── audio/       symphonia decode + cpal output + mixer + crossfade + stream_cache + http_source
├── library/     SQLite, local scanner (jwalk+lofty), FTS5 search, artwork, waveform
├── sources/     SourceProvider trait + local / ytmusic (Innertube) / spotify (rspotify PKCE) + resolver
├── services/    lyrics, sponsorblock, scrobbler, discord-rpc — Event consumers, feature-flagged
├── platform/    MPRIS / SMTC / MPNowPlayingInfoCenter, media keys, tray, paths
├── jam/         session layer + JamTransport trait + loopback/lan/p2p/cast-receiver impls
├── engine/      orchestrates {audio, library, sources, services, platform, jam} behind Command/Event
├── plugin-api/  stable types (Phase 10+ of old plan — seams only)
├── cli/         headless test harness — your primary UI until Phase 5
└── app/         GPUI frontend
```

**Dependency rule (enforce with a dependency-direction test):**
`app → engine → {core, audio, library, sources, services, platform, jam}`; `core` depends only on `serde`/`std`-ish; providers never import each other.

**Key crate choices:** `reqwest`(rustls) · `rusqlite`(bundled) · `symphonia`(features: `mkv` for WebM/Opus, `mp4`, `ogg`) · `cpal` + `rtrb` + `rubato` · `rspotify` (PKCE) · `lofty` · `jwalk` · `mdns-sd` · `tokio-tungstenite` · `webrtc` (pure-Rust Pion port) · `snow` (Noise E2E) · `blake3` · `wiremock` (dev, mock APIs)

---

## 3. Data model changes (do in Phase 0 — retrofits are expensive)

`crates/engine/src/api.rs` exists with Command/Event + `PeerId`/`origin`/versions. Evolve the models:

```rust
pub enum TrackSource {
    Local(PathBuf),
    YouTubeMusic(VideoId),     // resolved on demand via Innertube
    Spotify(SpotifyTrackId),   // resolved to YT via resolver+ISRC
}
pub struct Track {
    pub id: TrackId,                       // "src:provider:provider_id" — globally unique
    pub title: String,
    pub artists: Vec<String>,
    pub album: Option<String>,
    pub duration: Duration,
    pub source: TrackSource,
    pub isrc: Option<String>,              // canonical cross-provider key
    pub artwork: Option<ArtworkRef>,
    pub content_hash: Option<Blake3>,      // local files: decoded-audio hash (Jam Mode B later)
}
pub struct ResolvedAudio {                 // Spotube sourceMatch pattern
    pub stream_url: String, pub codec: Codec, pub bitrate: u32,
    pub expires_at: Instant,               // YT URLs expire ~6h — cache accordingly
}
```

New Commands: `ResolveAndPlay(TrackId)`, `Login { provider }`, `ImportPlaylist { provider, id }`. New Events: `Resolved(TrackId)`, `LyricsChanged`, `AuthStateChanged { provider }`, `JamSessionChanged`.

---

## 4. Phases

### PHASE 0 — Foundation (extend existing, ~2 days)
Already exists: workspace, api.rs, release profile. Add: model changes above · `SourceProvider` trait in `sources` (no impls) · `tracing` everywhere · CI 3-platform if not done · dependency-direction test.

### PHASE 1 — Audio Engine (the hard phase — engine-first per user)
Old plan's Phase 1 stands (symphonia→rtrb→cpal, 500ms buffer, allocation-free callback, seek=flush+reposition, position from samples-consumed). **Add for streaming:**
- `http_source.rs`: HTTP-range `Read + Seek` adapter over `reqwest` so symphonia can probe/decode remote Opus-in-WebM/AAC-in-MP4 — local files and streams go through the **same** decode pipeline
- `stream_cache.rs`: ViVi/SimpleCache pattern — write-through chunk cache, replay = disk
- `crossfade.rs`: dual decode pipelines + equal-power mixer (SimpMusic pattern), pre-decode next track for gapless
- ReplayGain gain staging in mixer
- **Tests:** old plan's corpus + broken files + seek storm · mock HTTP server (`wiremock`) serving ranged Opus → assert glitch-free playback + cache hit on replay · crossfade boundary click test · device-disconnect recovery
- **Done when:** `cli play <path|url>` plays local + ranged-HTTP audio gapless with crossfade, zero underruns in soak

### PHASE 2 — Unified Library (multi-source from start)
- SQLite schema: `tracks(source, isrc, ...)` · `albums` · `artists` · `playlists` · `playlist_tracks` · **`source_matches(track_key, source_slug, video_id, score, fetched_at)`** (Spotube cache) · `stream_cache_meta` · FTS5 over title/artist/album
- Local scanner: `jwalk` walk + `lofty` tags + mtime/size incremental rescan + `blake3` decoded-audio hash
- Artwork: embedded + remote fetch → 64/256/640px WebP thumbs, content-hash dedupe, LRU
- `waveform.rs`: peak extraction off-thread, cached
- **Tests:** old plan's fixture/rescan/search tests + 10k-file benchmark <60s · schema migration test

### PHASE 3 — Sources: YTM auth + Spotify + resolver (user's "yt auth + library population")
- `sources/ytmusic/`: Innertube client — visitor-data bootstrap → cookie/`SAPISIDHASH` auth → `search`, `browse` (home, artist, album, playlist), `next` (radio/related), `player` (stream manifest → best Opus itag). Port endpoint shapes from `reference/SimpMusic/core/service/kotlinYtmusicScraper` and `reference/vivi-music/innertube`
- `sources/spotify/`: `rspotify` OAuth PKCE loopback redirect → me/library, playlists, saved albums/tracks, search, ISRC on every track
- `sources/resolver.rs`: `resolve(Track) -> ResolvedAudio`:
  1. `source_matches` cache hit & fresh → return
  2. ISRC present → YTM `search(isrc)`; else `search("{title} {artist} official audio")`
  3. `core::matching::score()` — artist-in-channel, official regex, duration delta (±3s tolerance)
  4. persist match + stream URL with expiry
- Library population: import YTM + Spotify liked songs/playlists/albums into unified SQLite as `TrackSource::*` rows (no audio fetched at import — resolve lazily at play time)
- **Tests:** `wiremock` Innertube/Spotify fixtures from captured JSON · matcher property tests (fuzz titles/durations) · cache-hit avoids network · expired URL re-resolves · **manual:** `cli login ytm && cli import && cli play "playlist:X"` works end-to-end
- **Done when:** search across local+YTM+Spotify in one CLI result set; play any result gaplessly

### PHASE 4 — Engine integration + CLI (old plan Phase 3, extended)
Queue over mixed-source tracks · auto-advance with background resolve-ahead (resolve next track while current plays) · persist queue/position/volume · full CLI: `login/import/search/play/queue/status/seek`. **Tests:** scripted mixed-source scenarios, shuffle-seed reproducibility, restart persistence, 200-track soak with RSS flat.

### PHASE 5–6 — GPUI shell + core UI
Old plan Phases 4–5 stand. Layout mirrors the reference apps: sidebar · main content · persistent now-playing bar. Unified views (chips to filter source) · YTM home/browse pages · Spotify playlists · grouped instant search across sources · queue panel with per-track source badge. Image pipeline + virtualized lists per old plan. **Perf gates:** 10k scroll p99 <8ms, <150MB RSS after 50 albums, <100ms search.

### PHASE 7 — Services layer (feature-flagged, SimpMusic `-empty` pattern → Cargo features)
Each is an Event consumer behind `engine.subscribe()` — zero engine changes:
- `lyrics.rs`: LRCLIB → (fallback chain like SimpMusic) → returns word-level timings when available (ViVi karaoke model), synced scroll
- `sponsorblock.rs`: on `Resolved(yt_id)` fetch segments; engine position monitor skips `[start,end)` — skip boundary = seek, reuses tested seek path
- `scrobbler.rs`: Last.fm — scrobble at 50% or 240s (Spotube rule), MD5 `api_sig`
- `discord_rpc.rs`: now-playing presence (feature `discord`)
- **Tests:** mock provider servers; SponsorBlock skip lands within 50ms of segment start; scrobble fires exactly once

### PHASE 8 — Polish & effects
Album-art dynamic palette (all 4 apps), backdrop blur cached, transitions, marquee, context menus, drag-drop reorder, light/dark themes, settings page, parametric EQ + tempo. Re-run idle-CPU and memory-flat gates.

### PHASE 9 — Platform integration
Old plan Phase 7 stands: MPRIS (`zbus`) / SMTC (`windows-rs`) / `MPNowPlayingInfoCenter`, media keys, single-instance, tray, sleep-prevention.

### PHASE 10 — Jam (pluggable transports, P2P-first)
Session/control plane per old plan Phase 9 (host-authoritative, roles, rate-limit, E2E `snow`, 6-char codes, full-snapshot reconnect, `PROTOCOL_VERSION` reject). **Transport layer per user's direction:**

```rust
#[async_trait]
pub trait JamTransport: Send + Sync {
    async fn advertise(&self, s: &SessionInfo) -> Result<()>;
    async fn discover(&self) -> Result<Vec<SessionInfo>>;
    async fn join(&self, code: &str) -> Result<PeerChannel>;
    async fn send(&self, to: PeerId, msg: &WireMessage) -> Result<()>;
    fn incoming(&self) -> Receiver<(PeerId, WireMessage)>;
    fn capabilities(&self) -> TransportCaps;   // latency class → Mode A/B decision
}
```

Implementations, in build order:
1. **`loopback`** — two engines in one process/channel pair. Exists purely so every session-layer test runs without network. This is the "pluggable for testing" requirement.
2. **`lan`** — `mdns-sd` (`_rusticjam._tcp.local`) + `tokio-tungstenite`. Zero infrastructure — Spotify-Connect-equivalent on home WiFi.
3. **`p2p`** — `webrtc` data channels + STUN, code-pairing signaling through the smallest possible stateless rendezvous (one POST/GET pair; self-hostable in ~100 lines). E2E key via ECDH on the code before signaling sees anything.
4. **`cast-receiver`** — receiver mode: any device (or a headless `cli --receiver` process — a Raspberry Pi plugged into speakers) advertises itself as a playback target; senders pair by code and drive it via the same Command/Event wire protocol. Doubles as the **fallback path when WebRTC negotiation fails** (relayed WS over the rendezvous), which is exactly the YouTube-Cast-receiver model, self-hosted.

Mode A (collaborative queue/remote control) ships first on all transports; Mode B (sync playback) layered on `p2p`/`lan` only when `TransportCaps.latency_class` allows — clock-sync and drift-resampling per old plan (SNTP-style offset, min-RTT window, ±0.05% `rubato` resample, never drop samples).
- **Tests:** loopback for all session logic · `tc netem` jitter suite · guest mid-track join · host drop → clean "session ended" · 8-guest flood convergence · WebRTC-fail → cast-receiver fallback transparent to UI

### PHASE 11 — Packaging
Old plan Phase 8 stands (AppImage+Flatpak / MSI / .app, first-run flow, 8h soak).

---

## 5. Performance rules (unchanged, from ui-framework-decision.md)
No frame without dirty state · progress ticks 1Hz (60fps only while dragging) · cached blur · stop rendering occluded · audio callback: no alloc/lock/log · decode+images off-thread · bounded LRUs · `tracy` from day one · track p99 frame time and input-to-photon.

## 6. Risk register (new entries only)
| Risk | Mitigation |
|---|---|
| Innertube endpoint drift (YouTube changes API) | All Innertube JSON behind one `ytmusic` crate with versioned context payloads; `wiremock` fixtures detect breakage in CI; keep `yt-dlp` binary as emergency fallback resolver behind the same trait |
| Spotify rate limits | Cache aggressively (SQLite), batch library import, exponential backoff |
| WebM/Opus seek edge cases in symphonia | Pre-buffer full track to `stream_cache` when seek storm detected; seek = flush + reposition already tested |
| `webrtc` crate maturity | Transport trait isolates it; LAN + cast-receiver work without it; ship Jam v1 even if p2p slips |
| Dual-provider identity mess (same song twice) | ISRC primary key + `source_matches` unification + UI dedupe by (isrc|title+artist+duration±2s) |

## 7. First week, concretely
1. Land the Phase 0 model changes (TrackSource, ISRC, ResolvedAudio) + `SourceProvider` trait skeleton
2. `audio/http_source.rs`: ranged-HTTP reader + symphonia probe of a remote Opus stream
3. `stream_cache.rs` chunk cache with replay test
4. Dual-pipeline crossfade playing two local FLACs seamlessly
5. `cli play <path>` and `cli play <http-url>` both gapless

Item 1 is what makes everything else cheap. Items 2–4 are the riskiest audio work, front-loaded per engine-first strategy.
