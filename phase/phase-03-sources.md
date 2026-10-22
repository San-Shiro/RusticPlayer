# Phase 03 — Sources (YTM + Spotify) + UI Organisms

**Tracks:** A (E3) · B (U2) · **the "yt auth + library population" phase**

## Objective
Log into YouTube Music and Spotify, import both libraries into the unified DB, resolve any track to a playable stream (cached). UI kit gains all organisms including the virtualized 10k-row TrackList.

## Track A — Engine (sources)
Libraries: `reqwest` · `rspotify`(PKCE) · `wiremock`(dev). From scratch: Innertube client (port from `reference/SimpMusic/core/service/kotlinYtmusicScraper` + `reference/vivi-music/innertube`), resolver, matcher.

- [ ] `sources/ytmusic/` — visitor-data bootstrap → cookie + `SAPISIDHASH` auth → endpoints:
  - `search` (songs/albums/artists/playlists) · `browse` (home, artist, album, playlist) · `next` (radio/related) · `player` (adaptive manifest → best Opus itag; URL expiry ~6h)
- [ ] `sources/spotify/` — PKCE loopback OAuth, token refresh, me/library, playlists, saved albums/tracks, search (ISRC on every track)
- [ ] `core/matching.rs` — score: ISRC exact → artist-in-channel → `official\s(video|audio|music video|lyric video|visualizer)` regex → duration delta (±3s)
- [ ] `sources/resolver.rs` — cache check → ISRC search → score → persist `source_matches` + expiry
- [ ] Import: YTM liked/playlists + Spotify saved/playlists → unified rows; lazy audio resolve at play time; ISRC dedupe across providers
- [ ] Resolve-ahead: pre-resolve + prebuffer next queue item during current playback

**Tests:** wiremock fixtures (captured Innertube/Spotify JSON) · matcher proptest fuzz (titles, durations, unicode) · cache-hit = no network · expired URL re-resolves · token refresh flow · **manual: `cli login ytm` → `cli import` → `cli play playlist:X` end-to-end.**

## Track B — UI (organisms)
- [ ] NowPlayingBar (art/title-marquee/like ‖ transport ‖ waveform seek ‖ lyrics·queue·Jam toggles · volume · expand)
- [ ] TrackList — virtualized, sticky header, sort columns, 10k rows p99 <8ms (gpui-component List)
- [ ] CardGrid (responsive 5→3→2) · Sidebar (224px → 64px rail <900px) · QueuePanel (drag-reorder, insert-line indicator)
- [ ] ExpandedPlayer shell · LyricsPanel · SearchResults (grouped) · JamPanel · OnboardingDialog
- [ ] Gallery entries for all; perf capture in gallery with tracy

## Done when
Unified cross-source search in CLI; any result plays gaplessly; all organisms in gallery with perf gate green.
