# Core Engine — Detailed Phased Plan (Track A)

**Scope:** everything except pixels. Playback engine, streaming sources (YTM + Spotify), unified library, services, Jam networking. Consumed by any frontend through one `Command`/`Event` bus.
**Budgets:** <80MB RSS engine-only · ~0% CPU paused/idle · seek <50ms local, <300ms remote (cache miss) · resolve-ahead so track-to-track gap = 0.
**Rule zero:** `core` never does I/O. The audio callback never allocates, locks, or logs. Everything crosses the engine boundary as `Command`/`Event`.

---

## 1. Architecture

### 1.1 Crate map & dependency rule

```
app/cli/jam-guests ──► engine ──► core         (pure: models, queue, matching)
                        │─────► audio         (decode, output, mixer, cache)
                        │─────► library       (SQLite, scan, search, artwork, waveform)
                        │─────► sources       (ytmusic, spotify, resolver)
                        │─────► services      (lyrics, sponsorblock, scrobbler, discord)
                        │─────► platform      (MPRIS/SMTC, media keys)
                        └─────► jam           (session + transports)
```

Enforced by a dependency-direction test in CI (`engine` is the only crate allowed to import the others; providers never import each other).

### 1.2 Threading / runtime model

```
┌────────────────────────────────────────────────────────────┐
│ cpal audio callback thread  (real-time)                    │
│   reads rtrb ring ◄── mixer ◄── [ring A | ring B]          │
│   NO alloc / lock / log — volume via AtomicF32             │
├────────────────────────────────────────────────────────────┤
│ decode threads (2: current + next for gapless/crossfade)   │
│   symphonia decode → rubato resample → fill rings          │
├────────────────────────────────────────────────────────────┤
│ tokio runtime (net + timers)                               │
│   http_source ranged reads · stream_cache writer           │
│   Innertube/Spotify API · lyrics/sponsorblock/scrobbler    │
│   jam transports (ws/webrtc)                               │
├────────────────────────────────────────────────────────────┤
│ engine actor (single task, owns all state)                 │
│   Command in ──► mutate ──► broadcast Event (multi-consumer)│
└────────────────────────────────────────────────────────────┘
```

Single-actor engine = no shared mutable state, deterministic event order, trivially testable. Position is derived from **samples consumed by the callback**, never wall clock (required for Jam Mode B later).

### 1.3 Data flow — play a Spotify track

```
UI: PlayTrack("src:spotify:4uLU6...") 
  → engine: track from library (has ISRC)
  → resolver: source_matches cache? ──hit──► ResolvedAudio
                     │miss
                     ▼
              YTM search(ISRC) → matching::score() → persist match
  → audio: open http_source(url) → probe (WebM/Opus) → decode thread
  → callback consumes → PositionChanged events (1 Hz)
  → services on Resolved(video_id): sponsorblock fetch, lyrics fetch
  → pre-resolve next queue item in background (gapless)
```

---

## 2. Prebuilt libraries vs build-from-scratch

### Use prebuilt (do NOT reinvent)

| Area | Crate | Why |
|---|---|---|
| Decode | `symphonia` (features `mkv`, `mp4`, `ogg`, `flac`, `aac`) | WebM/Opus (YT streams) + all local codecs; pure Rust |
| Output | `cpal` | Cross-platform device abstraction |
| Ring buffer | `rtrb` | Lock-free SPSC, proven for audio callbacks |
| Resample | `rubato` (fft) | Needed for rate mismatch AND Jam drift correction later |
| HTTP | `reqwest` (rustls, no OpenSSL) | Ranged reads, API calls |
| DB | `rusqlite` (bundled) | FTS5 built in; no system dep |
| Tags | `lofty` | Pure-Rust read/write — Namida needed TagLib FFI, we don't |
| Dir walk | `jwalk` | Parallel, fastest scanner |
| Spotify API | `rspotify` (PKCE feature) | Mature OAuth + models; don't hand-roll |
| mDNS | `mdns-sd` | Pure Rust, async |
| WebSocket | `tokio-tungstenite` | Jam LAN/cast-receiver channels |
| WebRTC | `webrtc` (Pion port) | Jam P2P data channels; isolated behind trait if it disappoints |
| Crypto | `snow` (Noise), `blake3` | Jam E2E; content hashing |
| Images | `image`, `webp` | Thumbnail pipeline |
| Async | `tokio` | Runtime |
| Observability | `tracing`, `tracy` (feature `profiling`) | Perf gates from day one |
| Mock HTTP (dev) | `wiremock` | Deterministic Innertube/Spotify/SponsorBlock tests |

### Build from scratch (this is the product)

| Module | Why no library exists / why ours |
|---|---|
| `sources/ytmusic` Innertube client | No maintained pure-Rust Innertube crate. Port endpoint shapes from `reference/SimpMusic/core/service/kotlinYtmusicScraper` + `reference/vivi-music/innertube`. Plain JSON POSTs to `youtubei/v1/{search,browse,next,player}` with context payload. ~1.5k lines |
| `core/matching.rs` scorer | Spotube's `rankResults` is app code, not a lib. Ours: ISRC exact → artist-in-channel → official-regex → duration delta. Pure, fuzz-tested |
| `audio/crossfade.rs` mixer | SimpMusic pattern (dual players) adapted: two decode pipelines, equal-power ramp in callback-adjacent mixer |
| `audio/stream_cache.rs` | ExoPlayer `SimpleCache` equivalent doesn't exist for Rust. Chunk map (64KB blocks) + sparse file per (video_id, itag) |
| `audio/http_source.rs` | `Read+Seek` adapter over ranged reqwest for symphonia `MediaSource` |
| `library/waveform.rs` | Peak extraction is 100 lines on decoded samples; cache `[u8; 256]` per track |
| `services/*` thin REST clients | LRCLIB / SponsorBlock / Last.fm are trivial REST+JSON; MD5 `api_sig` signing for Last.fm |
| `services/lyrics` LRC+word parser | LRC line sync is trivial; word-level (enhanced LRC/A2 extension) parse ourselves — ViVi karaoke model |
| `jam/session.rs` | Host-authoritative session semantics are our design |
| `jam/transports/*` | loopback / lan / p2p / cast-receiver behind `JamTransport` |
| Engine actor + queue/shuffle | Core product logic; Fisher-Yates with stored seed |

---

## 3. Phases (Track A)

### E0 — Contracts & scaffolding (2–3 days)
**Build:** master-plan §3 model changes (`TrackSource`, ISRC, `ResolvedAudio`) in `engine/api.rs` · new Commands/Events (`ResolveAndPlay`, `Login`, `ImportPlaylist`, `Resolved`, `AuthStateChanged`, `LyricsChanged`, `JamSessionChanged`) · `SourceProvider` + `JamTransport` trait skeletons (no impls) · `sources`, `services`, `jam` crate stubs · tracing · dependency-direction test.
**Freeze point:** after E0 the Command/Event JSON is the **contract with Track B (UI)** — additive changes only, versioned by `API_VERSION`.
**Done when:** workspace builds, stubs pass tests, contract file exported to `plans/engine-api-contract.md`.

### E1 — Audio core (weeks 1–3, hardest)
**Build:**
- `audio/source.rs` — `MediaSource` abstraction: `FileSource` + `HttpSource` (ranged, 64KB blocks, retry+backoff)
- `stream_cache.rs` — write-through chunk cache; `GET` served from disk when present; meta table in SQLite
- `decoder.rs` — symphonia probe/decode → f32 interleaved; `rubato` resample to device rate
- `output.rs` — cpal stream, rtrb ring (500ms), AtomicF32 volume with 5ms smoothing ramp, underrun counter → silence, never glitch-loop
- `transport.rs` — `Stopped|Playing|Paused|Buffering`, seek = flush ring + reposition decoder; position from samples consumed
- `crossfade.rs` — pipeline B pre-decodes next track; equal-power ramp `cos/sin` over N ms; also gives true gapless
- ReplayGain: parse tags → gain staging in mixer
**Tests:** codec corpus + broken-file corpus (no panics, `proptest` garbage) · allocation-panicking global allocator on callback thread · 100-seek storm · gapless two-tone click test · `wiremock` ranged-Opus playback + cache-hit replay with server killed · device disconnect recovery.
**Done when:** `cli play <path|url>` local + HTTP, gapless, crossfade, 10-min glitch-free.

### E2 — Library (weeks 4–5)
**Build:** SQLite schema (tracks/albums/artists/playlists/playlist_tracks/**source_matches**/stream_cache_meta + FTS5) · migrations · scanner (jwalk + lofty, mtime/size incremental, blake3 decoded-audio hash) · artwork pipeline (embedded+remote → 64/256/640 WebP, content-hash dedupe, bounded LRU) · `waveform.rs` · search (FTS5, accent/case-insensitive).
**Tests:** 50-file fixture (unicode, multi-disc, compilations, no tags) · rescan-unchanged = zero reads · one-tag-change = one row update · delete removes · search unicode/partial · 10k scan <60s bench · thumbnail dedupe.
**Done when:** `cli scan/search` on 10k local files, instant search.

### E3 — Sources: YTM + Spotify + resolver (weeks 6–8)
**Build:**
- `sources/ytmusic/` — visitor bootstrap → cookie+`SAPISIDHASH` auth → `search`, `browse` (home/artist/album/playlist), `next` (radio), `player` (adaptive manifest → best Opus itag, URL expiry ~6h)
- `sources/spotify/` — rspotify PKCE loopback auth, token refresh, me/library, playlists, albums, search (ISRC on every track)
- `sources/resolver.rs` — cache → ISRC search → `matching::score()` → persist + expiry
- Library import: YTM + Spotify liked/playlists/albums → unified rows (lazy audio resolve)
- Dedupe: same ISRC across providers = one library entry, multiple `TrackSource`s
**Tests:** wiremock fixtures (captured Innertube/Spotify JSON) · matcher property tests · cache-hit avoids network · expired URL re-resolve · auth refresh flow · **manual:** `cli login ytm && cli import && cli play playlist:X`.
**Done when:** unified cross-source search in CLI; any result plays gaplessly.

### E4 — Engine integration + CLI (week 9)
**Build:** queue over mixed sources · resolve-ahead (resolve+prebuffer next during current) · persist queue/position/volume/shuffle · full CLI (`login/import/search/play/queue/status/seek`).
**Tests:** scripted mixed-source scenarios · shuffle seed reproducibility · restart persistence · **200-track mixed soak, RSS flat ±10%.**
**Done when:** CLI is a usable daily driver.

### E5 — Services (week 10)
Feature-flagged Event consumers (`--features lastfm,discord`):
`lyrics.rs` (LRCLIB first, word-timings when present) · `sponsorblock.rs` (fetch on `Resolved`, position monitor → seek-skip) · `scrobbler.rs` (50%/240s rule, MD5 api_sig) · `discord_rpc.rs`.
**Tests:** mock servers · skip lands <50ms · scrobble fires exactly once per track.

### E6 — Platform (week 11)
MPRIS (`zbus`) / SMTC (`windows-rs`) / `MPNowPlayingInfoCenter` behind one trait · media keys · single-instance · sleep-inhibit.
**Tests:** `playerctl` round-trip; OS widgets show art+position; second instance focuses first.

### E7 — Jam session layer (weeks 12–13)
**Build:** `jam/session.rs` — host-authoritative, roles (Host/Guest/Listener), 6-char codes, ECDH→`snow` E2E, rate limits, full-snapshot reconnect, `PROTOCOL_VERSION` gate. Transport `loopback` first (in-process pair) → `lan` (mdns-sd + tungstenite).
**Tests:** all session logic on loopback · wrong-code reject + join rate-limit · host drop → clean end · 8-guest flood convergence.

### E8 — Jam transports: P2P + cast-receiver (weeks 14–16)
**Build:** `p2p` (`webrtc` data channels + STUN + minimal self-hostable rendezvous: POST offer/GET answer, E2E key derived from code before signaling) · `cast-receiver` (receiver mode: headless `cli --receiver` advertises via code, senders drive it over the same wire protocol; **fallback when WebRTC fails** — relayed WS via rendezvous) · Mode A on all transports; Mode B only when `TransportCaps.latency_class == Low` (clock sync per development-plan: min-RTT window, ±0.05% rubato drift).
**Tests:** two machines LAN · WebRTC-fail → cast-receiver transparent fallback · `tc netem` 50/100/200ms jitter · mid-track join syncs <2s (Mode B).

### E9 — Hardening (week 17)
8h soak · RSS/idle-CPU gates · error taxonomy pass (`thiserror` everywhere, no user-facing panics) · crash reporting opt-in · packaging support (headless `cli --receiver` systemd unit).

---

## 4. Interface contracts (freeze in E0)

```rust
// sources/src/provider.rs
#[async_trait]
pub trait SourceProvider: Send + Sync {
    fn slug(&self) -> &'static str;                     // "ytmusic" | "spotify" | "local"
    async fn login(&self, flow: AuthFlow) -> Result<AuthState>;
    async fn search(&self, q: &str, limit: usize) -> Result<Vec<Track>>;
    async fn library(&self) -> Result<Vec<Playlist>>;   // liked songs = synthetic playlist
    async fn resolve(&self, t: &Track) -> Result<ResolvedAudio>; // local: trivial
}

// engine surface (unchanged shape): Engine::send(Command) / Engine::subscribe() -> Receiver<Event>
// jam/src/transport.rs — see master plan §Phase 10
```

**Testing pyramid:** `core` unit+proptest (fast, huge coverage) → crate-level integration with wiremock/fixtures → engine scenario tests (the crown jewels) → soak/manual gates per phase. Every phase: idle-CPU check + memory-flat soak before merge.
