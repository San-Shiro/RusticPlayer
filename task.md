# Rustic Player — Parallel Execution Tasks

Two tracks developed in parallel. **Contract:** `crates/engine/src/api.rs` (Command/Event) frozen at milestone **M0** — additive changes only, bump `API_VERSION`. UI never imports engine internals; engine never renders.
Details: `plans/core-engine-plan.md` (Track A) · `plans/ui-plan.md` (Track B) · `plans/rustic-player-master-plan.md` (strategy).

## Sync points

| # | When | What |
|---|---|---|
| S0 | M0 done | Contract freeze → exported to `plans/engine-api-contract.md`; UI starts against it |
| S1 | M2 engine / M3 ui | MockEngine (in `app/`) implements contract exactly — integration dry-run |
| S2 | M4 both | Real engine + real UI wired (daily-driver) |
| S3 | M5 engine | Jam session layer ready → Jam panel integrates |
| S4 | M6 both | Release candidate gates: 8h soak, <150MB, ~0% idle, p99 <8ms |

---

## TRACK A — Core Engine

### M0 · Contracts & scaffolding (E0)
- [x] Model changes in `api.rs`: `TrackSource`, ISRC, `ResolvedAudio`, new Commands/Events
- [x] `SourceProvider` + `JamTransport` trait skeletons; `sources`/`services`/`jam` crate stubs
- [x] tracing everywhere; dependency-direction test; export contract doc → **S0**

### M1 · Audio core (E1)
- [ ] `audio/source.rs`: FileSource + HttpSource (ranged, 64KB blocks)
- [ ] `stream_cache.rs`: write-through chunk cache + SQLite meta
- [ ] `decoder.rs`: symphonia (mkv/mp4/ogg/flac/aac) → f32 + rubato resample
- [ ] `output.rs`: cpal + rtrb 500ms ring + AtomicF32 volume ramp + underrun counter
- [ ] `transport.rs`: state machine, seek, samples-consumed position
- [ ] `crossfade.rs`: dual pipeline, equal-power ramp, gapless pre-decode
- [ ] ReplayGain in mixer
- [ ] Gates: corpus+proptest no-panic · alloc-free callback · seek storm · wiremock HTTP playback + cache replay

### M2 · Library (E2)
- [ ] SQLite schema + migrations (incl. `source_matches`, FTS5)
- [ ] Scanner: jwalk + lofty, mtime/size incremental, blake3 audio hash
- [ ] Artwork pipeline: 64/256/640 WebP, content-hash dedupe, bounded LRU
- [ ] `waveform.rs` peak cache
- [ ] Gates: fixture suite · rescan zero-reads · 10k scan <60s

### M3 · Sources (E3)
- [ ] `sources/ytmusic`: Innertube client — visitor bootstrap, cookie/SAPISIDHASH auth, search/browse/next/player
- [ ] `sources/spotify`: rspotify PKCE, refresh, library/playlists/search
- [ ] `sources/resolver.rs`: cache → ISRC → matching::score() → persist+expiry
- [ ] `core/matching.rs` + proptest fuzz
- [ ] Import pipelines → unified rows, ISRC dedupe
- [ ] Gates: wiremock fixtures · cache-hit-no-network · manual login→import→play

### M4 · Engine integration + CLI (E4) → **S2**
- [ ] Mixed-source queue, resolve-ahead, persistence
- [ ] CLI: login/import/search/play/queue/status/seek
- [ ] Gates: scenario tests · shuffle seed · restart restore · 200-track soak RSS flat

### M5 · Services + Platform + Jam session (E5–E7) → **S3**
- [ ] lyrics (word timings) · sponsorblock · scrobbler · discord (feature flags)
- [ ] MPRIS/SMTC/MediaCenter, media keys, single-instance
- [ ] jam/session.rs + loopback transport, then lan (mdns-sd + tungstenite)

### M6 · Jam transports + hardening (E8–E9) → **S4**
- [ ] p2p (webrtc + rendezvous, code-derived E2E)
- [ ] cast-receiver mode + WebRTC-fail fallback
- [ ] 8h soak · idle/RSS gates · error taxonomy

---

## TRACK B — UI

### M0 · Kit scaffold (U0) — starts after S0
- [ ] `crates/ui-kit` crate (gpui + gpui-component only, no engine import)
- [ ] Design tokens (palette/type/spacing/radii/elevation/motion) + dark/light
- [ ] Gallery example harness

### M1 · Atoms + molecules (U1)
- [ ] Atoms: Button, IconButton, SeekBar(+waveform), VolumeSlider, Marquee, Artwork, SourceChip, Skeleton, TextField, ContextMenu, …
- [ ] Molecules: TrackRow, AlbumCard, ArtistCard, PlaylistCard, TransportControls, QueueItem, LyricsLine, SectionHeader
- [ ] All states (hover/pressed/disabled/loading/empty) in gallery

### M2 · Organisms (U2)
- [ ] NowPlayingBar, TrackList (virtualized), CardGrid, Sidebar, QueuePanel
- [ ] ExpandedPlayer, LyricsPanel, SearchResults, JamPanel, OnboardingDialog
- [ ] Gate: 10k-row list p99 <8ms

### M3 · Shell + MockEngine (U3) → **S1**
- [ ] App shell: sidebar/main/right-panel/now-playing-bar, router, responsive breakpoints (1100/900/700px)
- [ ] EngineBridge + MockEngine (scripted events, fake latency)
- [ ] Full click-through of desktop flow map

### M4 · Pages (U4) → **S2**
- [ ] Onboarding, Home, Search, Library (4 tabs), Album/Artist/Playlist detail, Settings, Jam page
- [ ] Every empty/loading/error state

### M5 · Real integration (U5)
- [ ] Swap MockEngine → real EngineBridge; dynamic palette pipeline (off-thread extraction, 300ms crossfade)
- [ ] Daily-driver gate

### M6 · Panels, animation pass, polish (U6–U7) → **S4**
- [ ] Lyrics/queue/Jam panels on live data
- [ ] Motion tokens applied end-to-end; reduced-motion path
- [ ] Context menus, drag-drop reorder, marquee, media-key feedback
- [ ] Gates: idle ~0% CPU · <150MB · tracy transitions p99 <8ms · resize 600px→4K

---

### Mobile (deferred, design-only for v1)
- [ ] Thin client consuming Command/Event protocol (FFI or Jam transport), per `plans/ui-plan.md` §6 screens/flows — schedule after S4
