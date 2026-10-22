# Phase 00 — Contracts & Foundation

**Tracks:** A (E0) · B (prep) · **Exit: S0 — contract freeze**

## Objective
Freeze the `Command`/`Event` API that both tracks build against for the entire project. Everything after this phase is additive-only, versioned by `API_VERSION`.

## Track A — Engine
- [ ] Evolve `crates/engine/src/api.rs`:
  - `TrackSource { Local(PathBuf), YouTubeMusic(VideoId), Spotify(SpotifyTrackId) }`
  - `Track { id: "src:provider:id", title, artists: Vec<String>, album, duration, source, isrc, artwork, content_hash }`
  - `ResolvedAudio { stream_url, codec, bitrate, expires_at }`
  - New Commands: `ResolveAndPlay(TrackId)`, `Login { provider }`, `ImportPlaylist { provider, id }`
  - New Events: `Resolved(TrackId)`, `AuthStateChanged { provider }`, `LyricsChanged`, `JamSessionChanged`
- [ ] `sources` crate stub + `SourceProvider` trait (`slug/login/search/library/resolve`) — no impls
- [ ] `jam` crate stub + `JamTransport` trait — no impls
- [ ] `services` crate stub
- [ ] `tracing` + `tracing-subscriber` wired in all crates; `profiling` feature → tracy
- [ ] Dependency-direction test in CI (`core` gains no deps; providers never import each other)
- [ ] CI green: build/test/clippy `-D warnings`/fmt on Linux + Windows + macOS

## Track B — UI prep
- [ ] Read the frozen contract; flag anything the UI can't express (last chance for changes)
- [ ] `crates/ui-kit` + `crates/app` crate skeletons compile against gpui pin (SHAs recorded in workspace Cargo.toml)

## Deliverable
- [ ] `plans/engine-api-contract.md` — the Command/Event JSON schema, exported and committed

## Done when
Workspace builds on 3 OSes; contract doc exists; **S0 declared** — UI track starts.
