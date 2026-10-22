# Phase 04 — Engine Integration + UI Shell & Pages

**Tracks:** A (E4) · B (U3–U4) · **Exits: S1 (MockEngine) + S2 (daily-driver)**

## Objective
Engine becomes a coherent product behind the CLI; UI becomes a complete click-through app against MockEngine — then the two connect with zero rework.

## Track A — Engine (integration + CLI)
- [ ] Mixed-source queue: local + YTM + Spotify tracks in one queue; auto-advance
- [ ] Shuffle: Fisher-Yates with stored seed (reproducible); repeat off/all/one boundaries
- [ ] Persistence: queue, position, volume, shuffle/repeat across restart
- [ ] Full CLI: `login` `import` `search` `play` `queue [list|add|next|clear]` `status` (live event stream) `seek`

**Tests:** scripted mixed-source scenario (scan→import→search→play→next→seek→pause→resume, assert event sequence) · same-seed shuffle = same order, all tracks once · restart mid-play restores queue+position · **200-track mixed soak: RSS flat ±10%.**

## Track B — UI (shell + pages)
- [ ] `EngineBridge` trait in `app/` + **MockEngine**: scripted Events, fake latency, deterministic — mirrors the frozen contract exactly → **S1**
- [ ] App shell: titlebar (⌘K palette) · sidebar · router outlet · right panel (docked >1100px, overlay below) · persistent NowPlayingBar
- [ ] Responsive breakpoints: 1100px (panel overlay) · 900px (icon rail) · 700px (hamburger)
- [ ] Pages with all states: Onboarding · Home (feed rows) · Search (chips + grouped results) · Library (Albums/Artists/Tracks/Playlists tabs + source filters) · Album/Artist/Playlist detail · Settings · Jam
- [ ] Navigation per flow map: play never navigates · bar-art toggles ExpandedPlayer · Alt+←/→ history

## Done when
CLI usable as a daily driver for a day (find 5 bugs, fix them); full UI click-through on MockEngine with every empty/loading/error state → **S1, S2 declared.**
