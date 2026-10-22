# Phase 05 — Services & Platform + Live UI Integration

**Tracks:** A (E5–E6) · B (U5)

## Objective
Real engine drives the real UI (daily-driver quality). Event-consumer services go live; OS integration makes it feel native.

## Track A — Engine (services + platform)
Feature flags (SimpMusic `-empty` pattern → Cargo features, default off): `lastfm` · `discord`.

- [ ] `services/lyrics.rs` — LRCLIB first → fallback chain; parse line-synced LRC + word-level timings (enhanced LRC) → `LyricsChanged` events
- [ ] `services/sponsorblock.rs` — on `Resolved(video_id)` fetch segments; engine position monitor seek-skips `[start,end)` (reuses tested seek path)
- [ ] `services/scrobbler.rs` — Last.fm: scrobble at 50% or 240s; MD5 `api_sig`; now-playing updates
- [ ] `services/discord_rpc.rs` — now-playing presence
- [ ] `platform/`: MPRIS (`zbus`) · SMTC (`windows-rs`) · `MPNowPlayingInfoCenter` behind one trait · media keys · single-instance (second launch focuses window) · sleep-inhibit during playback

**Tests:** mock provider servers · SponsorBlock skip lands <50ms after segment start · scrobble fires exactly once · `playerctl play-pause` round-trip; OS widgets show correct art+position.

## Track B — UI (live integration)
- [ ] Swap MockEngine → real EngineBridge (spawn engine on background executor; events forwarded to foreground; views dispatch Commands — one module touches the engine)
- [ ] Dynamic palette pipeline: extract dominant colors off-thread on track change → animate accent ramp 300ms
- [ ] Lyrics panel on live `LyricsChanged`: active-line recenter 250ms, word-by-word fill synced (linear, not eased)
- [ ] SponsorBlock indicator on seekbar (skipped ranges dimmed)
- [ ] Settings pages wired: accounts (auth status/re-login), library folders, audio (device/buffer/crossfade/ReplayGain), appearance, cache

## Done when
You replace your daily music player with it for a week. Lyrics, SponsorBlock, scrobbling, and OS media controls all work on Linux + Windows (+macOS smoke).
