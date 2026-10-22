# UI — Detailed Phased Plan (Track B)

**Scope:** every pixel. GPUI + gpui-component desktop app (Linux primary, Windows/macOS), built **component-first** so the kit is reusable in other projects. Mobile ships as a design spec (§6) for a future thin client over the same Command/Event wire protocol — GPUI has no mobile targets, so no mobile code in v1, but every layout decision is made mobile-aware.
**Budgets:** <150MB total RSS · ~0% idle CPU · p99 frame <8ms on 10k-track lists · startup to first pixel <150ms · keystroke→search-results <100ms.

---

## 1. Component-based strategy (reusable in other projects)

Two crates, hard separation:

```
crates/ui-kit/     PRESENTATION ONLY. Depends on gpui + gpui-component + serde.
                   No engine import, no app state, no I/O. Every component is
                   props-in / callback-out (on_click, on_seek, ...). Themable
                   via DesignTokens. Publishable standalone — this is the crate
                   you reuse in other projects.

crates/app/        APPLICATION. Pages, navigation, EngineBridge, view state.
                   Composes ui-kit components, feeds them engine data.
```

**Rule:** if a component knows what a `TrackId` is, it belongs in `app/`. `ui-kit` knows `String`, `Duration`, `f32`, `Image` — nothing else. (Exception: thin generic row-model traits for virtualized lists.)

### Design tokens (`ui-kit/src/tokens.rs`)
```rust
pub struct DesignTokens {
    pub colors: Palette,      // base, surface, elevated, accent, on_accent, muted, danger
    pub dynamic: Option<Palette>,  // derived from album art; overrides accent ramp live
    pub type_scale: TypeScale,     // display/headline/title/body/label/caption (px + weight)
    pub spacing: Spacing,          // 4pt grid: 4/8/12/16/24/32/48
    pub radii: Radii,              // sm 4 · md 8 · lg 12 · xl 20 · full
    pub elevation: Elevation,      // shadow1..3, blur_material (cached backdrop blur)
    pub motion: Motion,            // §5 — durations/easings/springs as data
}
```
Dark + light base palettes; dynamic palette recomputed off-thread on track change (Namida/Spotube/vivi pattern), crossfaded over 300ms.

---

## 2. Component inventory (ui-kit)

**Atoms** (`atoms/`): `Button`, `IconButton`, `SeekBar` (waveform-aware variant), `VolumeSlider`, `Marquee` (overflow scroll text), `Artwork` (rounded, placeholder shimmer, crossfade on src change), `SourceChip` (Local/YTM/Spotify badge), `Skeleton`, `Badge`, `Switch`, `TextField` (search variant w/ debounce hook), `Tooltip`, `ContextMenu`, `Avatar`.

**Molecules** (`molecules/`): `TrackRow` (art 40px · title marquee · artist · source chip · duration · hover actions: play-next/queue/like/⋮), `AlbumCard` (art 1:1 · title · artist · hover play overlay), `ArtistCard` (circular art), `PlaylistCard`, `TransportControls` (prev/play-pause/next · shuffle · repeat tri-state), `QueueItem` (drag handle · attribution avatar for Jam), `LyricsLine` (word-by-word highlight segments), `SectionHeader` (title + "show all").

**Organisms** (`organisms/`): `NowPlayingBar` (§4.3), `TrackList` (virtualized, 10k rows, sticky header, multi-sort), `CardGrid` (responsive 5→3→2), `Sidebar` (nav + playlists, collapsible to icons <900px), `QueuePanel` (drag-reorder, drop indicator line), `ExpandedPlayer` (§4.4), `LyricsPanel` (synced scroll, karaoke word fill), `SearchResults` (grouped tracks/albums/artists), `JamPanel` (participants, code, QR), `OnboardingDialog` (provider login cards).

Each component ships: default/hover/pressed/disabled/loading/empty states + a gallery entry (`ui-kit/examples/gallery.rs`) — the gallery doubles as visual regression surface and as docs for reuse in other projects.

---

## 3. Desktop structure

```
┌──────────────────────────────────────────────────────────────────┐
│ titlebar (custom: window controls · breadcrumb · ⌘K palette)     │
├───────────┬──────────────────────────────────────────┬───────────┤
│ SIDEBAR   │  MAIN CONTENT (router outlet)            │ RIGHT     │
│ 224px     │                                          │ PANEL     │
│ ┌───────┐ │  Home / Search / Library / Detail pages  │ 320px     │
│ │Home   │ │                                          │ (toggle)  │
│ │Search │ │                                          │ Queue or  │
│ │Library│ │                                          │ Lyrics or │
│ ├───────┤ │                                          │ Jam       │
│ │PLAYL. │ │                                          │           │
│ │ + New │ │                                          │           │
│ │ • Gym │ │                                          │           │
│ └───────┘ │                                          │           │
│  [Jam] [⚙]│                                          │           │
├───────────┴──────────────────────────────────────────┴───────────┤
│ NOW PLAYING BAR (72px) — persistent, always visible              │
└──────────────────────────────────────────────────────────────────┘
```

Responsive: <1100px right panel overlays instead of docking · <900px sidebar → 64px icon rail · <700px sidebar hidden behind hamburger (mobile layout logic, §6).

### 3.1 Now Playing Bar contents
`[Artwork 48px]` · `title (marquee) / artist` · `♥ like` ‖ `shuffle prev ⏯ next repeat` · `position / waveform-seekbar / duration` ‖ `lyrics toggle` · `queue toggle` · `Jam cast button` · `volume slider` · `expand ⤢`
Waveform seekbar uses cached peaks (Namida); fallback flat bar until computed.

### 3.2 Expanded Player (full-window overlay, slides up 350ms spring)
Left: large artwork (3D tilt on pointer, vivi/flutter_tilt-style) + optional canvas loop later. Right: transport · waveform seek · volume · tabbed `Lyrics | Queue | Related (YTM radio)`. Background: cached backdrop blur material tinted by dynamic palette. Esc/⤡ collapses.

---

## 4. Pages — what each contains

| Page | Contents | Key states |
|---|---|---|
| **Onboarding** (first run) | provider login cards (YTM cookie import guide · Spotify OAuth button · local folder picker) → import progress | per-provider auth pending/failed |
| **Home** | YTM feed rows: "Quick picks" · "Recently played" · "Your mixes" (CardGrid rows, horizontal scroll) · Spotify recently played | skeletons · offline (cached only) |
| **Search** | debounced field · filter chips (All/Tracks/Albums/Artists/Playlists) · grouped `SearchResults`, each row shows `SourceChip` | no-results · per-source error chip |
| **Library** | tab bar: Albums · Artists · Tracks · Playlists · source filter chips · grid/list toggle · sort menu | empty (CTA to onboard/import) |
| **Album detail** | header: 256px art · title · artist · year · play/shuffle · source chip; `TrackList` w/ disc grouping; "More by artist" row | — |
| **Artist detail** | header art/banner · top tracks (5) · discography grid · YTM radio button | — |
| **Playlist detail** | header (art collage, owner, count) · `TrackList` w/ drag-reorder (own playlists) · attribution avatar per row when Jam | — |
| **Queue** (right panel) | "Now playing" + up-next `QueueItem`s · reorder/remove · save-as-playlist · clear | empty |
| **Lyrics** (right panel / expanded tab) | synced `LyricsLine`s, active line centered · word-by-word fill when timings exist · plain text fallback | "no lyrics found" |
| **Settings** | Accounts (per-provider auth status/re-login) · Library folders · Audio (device, buffer, crossfade ms, ReplayGain) · Appearance (theme, dynamic palette toggle) · Cache (sizes, clear) · About | — |
| **Jam** (right panel or /jam) | host: code + QR + participant list (roles, kick, lock queue) · guest: code entry + nearby sessions (mDNS) · connection status pill | reconnecting · session-ended · version-mismatch |
| **Local folders detail** | scanner progress bar · last-scan stats | scanning |

---

## 5. Desktop navigation flow map

```
                ┌─────────────┐
                │ Onboarding  │ (first run / missing auth)
                └──────┬──────┘
                       ▼
   ┌────────┐    ┌──────────┐    row click    ┌───────────────┐
   │ Sidebar├───►│   Home   ├────────────────►│ Album/Artist/ │
   │        │    └────┬─────┘                 │ Playlist detail│
   │        │         │ chip "show all"       └──────┬────────┘
   │        │    ┌────▼─────┐                        │ track click → play
   │        ├───►│  Search  │──result click──────────┤
   │        │    └──────────┘                        ▼
   │        │    ┌──────────┐  tab switch     ┌───────────────┐
   │        ├───►│ Library  ├────item────────►│  (details)    │
   │        │    └──────────┘                 └───────────────┘
   │        │    ┌──────────┐
   └────────┴───►│ Settings │
                 └──────────┘
 Global overlays (any page): ExpandedPlayer (⤢ / click bar art)
                             Queue | Lyrics | Jam right panel (toggle buttons)
                             ⌘K command palette (jump anywhere)
 Deep rules: play action NEVER navigates · clicking bar art toggles ExpandedPlayer
             · back/forward mouse buttons + Alt+←/→ navigate history
```

## 6. Mobile flow map & structure (design spec for future thin client)

Implementation path: a thin mobile client consumes the same versioned Command/Event protocol (local engine via FFI or Jam transport to a desktop host — Spotube-Connect-style). Screens reuse `ui-kit` **design tokens and component specs**, not GPUI code.

```
┌─────────────────────────┐   Structure per screen:
│  Bottom nav (56px)      │   Home: chip rows, vertical scroll (same data as desktop Home)
│ [Home][Search][Library] │   Search: full-width field, chips → grouped results
│  [Jam]                  │   Library: top tabs Albums/Artists/Tracks/Playlists
└─────────────────────────┘   Settings: single list, grouped (accounts/audio/cache)
         ▲
┌─────────────────────────┐   Mini player (56px, above bottom nav):
│ MINI PLAYER             │   art 40 · title/artist · ⏯ · next
│ (tap → expand, swipe ▲) │   swipe left/right on mini player = next/prev track
└─────────────────────────┘
         │ drag-up (spring, velocity-aware)
         ▼
┌─────────────────────────┐   Expanded player (full screen sheet):
│ artwork (1:1, tilt)     │   swipe-down dismisses with spring
│ title/artist (marquee)  │   background = blur material + dynamic palette
│ waveform seekbar        │   tabs: Lyrics | Up Next | Related
│ transport + shuffle/rep.│   Lyrics: same word-by-word component spec
│ ♥  source-chip  Jam     │   Up Next: drag-reorder, swipe-to-remove
└─────────────────────────┘
Flow: Onboarding → Home ⇄ Search ⇄ Library → Detail pages (push, back gesture)
      Mini player persistent across all tabs; Jam tab = join/host sheets
```

## 7. Animation system (`tokens.motion` — all values as data)

| Token | Value | Used for |
|---|---|---|
| `dur.instant` 100ms | ease-out | hover fades, chip toggles |
| `dur.fast` 180ms | ease-out | menu/popover open, row hover bg |
| `dur.med` 300ms | emphasized ease (`cubic-bezier(0.2,0,0,1)`) | page transitions (fade+8px rise), palette crossfade |
| `spring.sheet` | stiffness 260, damping 26 | expanded player, right panel, mobile sheets |
| `spring.press` | stiffness 400, damping 28 | button press scale 0.97 |
| `marquee` | 40px/s, 1.5s edge pause | overflowing titles |
| `lyrics.scroll` | 250ms ease-out | active line recenter |
| `lyrics.wordfill` | linear, tied to word timestamps | karaoke fill (NOT eased — synced) |

**Rules:** artwork crossfade 250ms on track change · page transitions only on route change, never on data update · seek bar ticks 1Hz (60fps only while dragging) · backdrop blur cached, invalidated on track change only · `prefers-reduced-motion` → all motion becomes 100ms crossfades · nothing animates when window occluded.

## 8. UI phases (Track B, parallel to engine)

| Phase | Weeks | Build | Gate |
|---|---|---|---|
| **U0** Kit scaffold | 1 | `ui-kit` crate, tokens, gallery harness, dark/light | gallery runs, theme switch live |
| **U1** Atoms+molecules | 2–3 | §2 atoms+molecules w/ all states | every state in gallery |
| **U2** Organisms | 4–5 | NowPlayingBar, TrackList, CardGrid, Sidebar, QueuePanel | 10k-row TrackList p99 <8ms in gallery |
| **U3** Shell + mock engine | 6 | app shell, router, `EngineBridge` trait + **MockEngine** (scripted Events, fake latency) | click-through of §5 map, zero real engine |
| **U4** Pages | 7–9 | §4 pages + Onboarding + Settings | all empty/loading/error states |
| **U5** Integration | 9–10 | swap MockEngine→real engine (needs E4) | daily-driver usable |
| **U6** Lyrics/queue/Jam panels + animations pass | 11–12 | §7 applied end-to-end | tracy: transitions p99 <8ms |
| **U7** Polish + platform chrome | 13 | context menus, dnd, marquee, palette, media-key feedback, tray | idle ~0% CPU, <150MB, perf gates |

**Sync points with Track A:** contract freeze (E0→U0 start) · MockEngine mirrors contract exactly (U3) · real integration at E4/U5 · Jam UI at E7+ (U6).
