# Rustic Player — UI Track B Kickoff: Design System U0 + U1 (Tokens → Gallery → Atoms → Molecules)

## Phase placement

This work is **Track B U0 + U1** — it maps to **Phase 01 Track B** (kit scaffold: tokens + gallery, per `phase/phase-01-audio-core.md`) and **Phase 02 Track B** (atoms + molecules). It is unblocked **now**: Phase 00 froze the Command/Event contract (S0), and Track B runs parallel to engine Phases 01–02. No engine code changes; the contract stays untouched.

## Design source of truth

- **Language:** the Lovable prototype (`reference/lovable/src-01-aug-2026/src/styles.css`) — *"noir lacquer + brass, dark by nature."* Its oklch values become our dark palette verbatim (converted to `gpui::Hsla`).
- **Structure/rules:** `plans/ui-plan.md` §1 (token shape), §2 (component inventory), §7 (motion as data). `plans/ui-framework-decision.md` §6 (performance rules) applies throughout.
- The `dynamic` palette slot (album-art accent override) is kept in the token model per ui-plan but stays `None` until Phase 04/05.

## Hard constraints (from repo docs, non-negotiable)

- `ui-kit` is **presentation-only**: no engine/core import, no I/O, no app state. Props-in / callback-out. It must never know what a `TrackId` is — rows take `SharedString`/`Duration`/indexes. (ui-plan §1 rule; enforced by the existing dependency-architecture test.)
- Budgets: p99 frame <8ms, ~0% idle CPU — **never render a frame when nothing is dirty**; animations stop at rest; `prefers-reduced-motion` path from day one.
- gpui stays behind the optional `gpui-ui` feature so engine/CLI CI builds never compile GPUI.

---

## Task 0 — Unblock the workspace (prerequisite)

`cargo build --workspace` currently fails: `crates/audio/Cargo.toml` dev-dep `assert-no-alloc = "0.1"` doesn't exist on crates.io. Rename to `assert_no_alloc = "0.1"` (underscores). Verify `cargo build --workspace` goes green before any UI work.

## Task 1 — Dependency wiring: gpui + gpui-component (the risky part)

**Problem found during planning:** our workspace pins `gpui` at `rev = "ce6f3af…"`, but `gpui-component` (checked `main`, v0.5.2) depends on `gpui = { git = zed }` *unpinned*. Two different rev specs of the same git repo = two gpui checkouts in the tree = type mismatches everywhere (`gpui::App` from rev A ≠ rev B). Independent rev pins cannot work.

**Solution — Cargo.lock is the pin:**
1. Workspace `Cargo.toml`: change `gpui` to unpinned git; add `gpui-component = { git = "https://github.com/longbridge/gpui-component" }` unpinned. Add a comment documenting: *"both track zed HEAD; the SHA pin lives in Cargo.lock; bump deliberately via `cargo update -p gpui --precise <sha>` (gpui and gpui-component must bump together)."*
2. Seed the lockfile at the Phase-0-chosen rev: `cargo update -p gpui --precise ce6f3af5f7ae2bbdb002c8ce5cc38e96179de811` (fall back to lock-resolved HEAD if gpui-component's code requires newer gpui — record the resulting SHAs in the comment).
3. Commit `Cargo.lock` (already untracked-pending; it must become a tracked, reproducible artifact from here on).
4. `crates/ui-kit/Cargo.toml`: add `gpui = { workspace = true, optional = true }`, `gpui-component = { workspace = true, optional = true }`, `serde` (token serialization later); extend feature: `gpui-ui = ["dep:gpui", "dep:gpui-component"]`.
5. Add per-target `gpui_platform` block to `crates/app` (pattern from `reference/.../desktop/Cargo.toml`): Linux → `features = ["font-kit", "x11", "wayland"]`; non-Linux → `features = ["font-kit"]`.
6. Gate: `cargo check -p ui-kit --features gpui-ui` compiles on this Windows machine.

**Note:** gpui-component uses `schemars 1.x`; our core uses 0.8. They coexist per-crate — no action needed, just don't unify them.

## Task 2 — Design tokens (`crates/ui-kit/src/tokens.rs`)

Full `DesignTokens` per ui-plan §1, with concrete values from the Lovable CSS (oklch → `gpui::hsla` conversion at definition site; keep the oklch source values in comments):

| Token group | Values |
|---|---|
| `Palette` (dark, canonical) | base `oklch(0.145 0 0)` · surface `oklch(0.205 0 0)` · elevated/surface-2 `oklch(0.25 0.004 90)` · accent/brass `oklch(0.73 0.11 88)` · on_accent `oklch(0.145 0 0)` · foreground/patina `oklch(0.88 0.086 90)` · muted_fg `oklch(0.66 0.048 90)` · danger `oklch(0.62 0.2 27)` · border = brass @ 20% alpha, input @ 28%, ring = brass |
| `Palette` (light) | derived inversion (marked *rough, refine in U6 polish*) — needed for the live theme-switch gate |
| `TypeScale` | body font `"Rubik"`, display/headings `"Space Mono"`; steps: display / headline / title / body / label / caption (px + weight); plus `label_caps` util (11px, 600, 0.14em tracking, uppercase — from Lovable `@utility label-caps`) |
| `Spacing` | 4pt grid: 4 / 8 / 12 / 16 / 24 / 32 / 48 |
| `Radii` | sm 4 · md 8 · lg 12 · xl 20 · xxl 24 (Lovable's `1.5rem` card radius) · full |
| `Elevation` | shadow1..3 (`gpui::BoxShadow` presets) + `blur_material` placeholder (cached backdrop blur, consumed by ExpandedPlayer at U2+) |
| `Motion` (ui-plan §7, as data) | `dur.instant` 100ms ease-out · `dur.fast` 180ms ease-out · `dur.med` 300ms `cubic-bezier(0.2,0,0,1)` · `spring.sheet` 260/26 · `spring.press` 400/28 (press scale 0.97) · marquee 40px/s + 1.5s edge pause · lyrics.scroll 250ms · lyrics.wordfill linear · `reduced_motion: bool` → everything degrades to 100ms crossfades |

Also `src/theme.rs`: `Theme { tokens, mode: Dark|Light, dynamic: Option<Palette> }`, stored as a gpui global; `apply()` maps our palette onto `gpui-component`'s `Theme` so wrapped primitives inherit Rustic styling; live toggle + reduced-motion flag.

## Task 3 — Assets

- **Fonts:** bundle Rubik (400/500/700) + Space Mono (400/700) TTFs in `crates/ui-kit/assets/fonts/`; register at app/gallery startup via the gpui text system; fallback chain `"Rubik", ui-sans-serif, system-ui` / `"Space Mono", ui-monospace` per Lovable CSS.
- **Icons:** Lucide SVGs (gpui-component ships no icon art — "SVGs added by you") in `crates/ui-kit/assets/icons/`; `src/icon.rs` maps an `Icon` enum → asset. Initial set (~24): play, pause, skip-back, skip-forward, shuffle, repeat, repeat-1, heart, heart-filled, plus, list-plus, more-horizontal, search, volume-x/1/2, music, disc, users (Jam), settings, chevron-right/down, x, check, folder.

## Task 4 — Gallery harness (`crates/ui-kit/examples/gallery.rs`)

Before any component: the harness every component lands in. Reference gpui-component's own examples (`hello_world`, `sidebar`) for the pinned-rev app-bootstrap API.
- Left nav: **Tokens** section (palette grid, type-scale specimen, spacing/radii/elevation/motion swatches) + **Atoms** + **Molecules** entries.
- Preview pane + top bar: dark/light toggle (live), reduced-motion toggle, state selector (default / hover / pressed / disabled / loading / empty where applicable — forced states via props; hover/pressed also live-interactive).
- Run: `cargo run -p ui-kit --example gallery --features gpui-ui`.
- **U0 gate:** gallery runs on Windows, theme switch live, token showcase renders the Lovable language.

## Task 5 — Atoms (`crates/ui-kit/src/atoms/`)

**Strategy — wrap before build.** First audit gpui-component's component list at the locked commit; where a primitive exists, wrap it with Rustic theming behind our API (contains upstream API drift in one layer). Expected mapping (verify at implementation):

| Atom | Approach |
|---|---|
| `Button` (primary brass / secondary / ghost / danger; sm/md/lg; loading spinner; press-scale 0.97 spring) | wrap gpui-component Button |
| `IconButton`, `Switch`, `TextField` (search variant + debounce hook), `Tooltip`, `ContextMenu`, `Badge`, `Skeleton` (shimmer), `Avatar` | wrap gpui-component equivalents if present, else build |
| `SeekBar` (flat + waveform variant taking `peaks: &[f32]`; `position`/`duration` as props, `on_seek` callback; 60fps only while dragging) | **custom** |
| `VolumeSlider` | **custom** (slider + icon state) |
| `Marquee` (40px/s, 1.5s edge pause; only animates on overflow; stops when occluded/reduced-motion) | **custom** |
| `Artwork` (rounded, shimmer placeholder, 250ms crossfade on src change) | **custom** |
| `SourceChip` (Local/YTM/Spotify badge, ui-kit-local `SourceKind` enum — not `TrackSource`) | **custom** |

Every atom: all six states reachable in gallery.

## Task 6 — Molecules (`crates/ui-kit/src/molecules/`)

Primitive-typed props only (no `TrackId`): e.g. `TrackRow { title: SharedString, artist: SharedString, duration: Duration, source: SourceKind, artwork: Option<Image>, …, on_play/on_play_next/on_queue/on_like/on_more }`.

- `TrackRow` — art 40px · title marquee · artist · SourceChip · duration · hover actions (play-next/queue/like/⋮)
- `AlbumCard` — 1:1 art · title · artist · hover play overlay (radius xxl 24 per Lovable)
- `ArtistCard` — circular art
- `PlaylistCard` — art collage slot
- `TransportControls` — prev / play-pause / next · shuffle · repeat tri-state (Off→All→One)
- `QueueItem` — drag handle · attribution-avatar slot (Jam, later)
- `LyricsLine` — word segments `Vec<(SharedString, start, end)>` + `progress` prop → linear karaoke fill (never eased — synced); plain-line fallback
- `SectionHeader` — title + "show all"

Every molecule: default/hover/disabled/loading/empty states in gallery. **U1 gate:** every atom + molecule with every state browsable in the gallery; dark and light.

## Task 7 — Verification (all must pass)

1. `cargo build --workspace` · `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo fmt --check`
2. `cargo test -p engine --test dependency_architecture_test` — proves ui-kit still has zero engine/core edges
3. Gallery manual checklist on this Windows machine: theme toggle live · reduced-motion toggle works · every atom/molecule × state renders · marquee pauses at edges · animations stop at rest (no idle repaint — sanity-check via window occluded → no frames)
4. CI matrix (ubuntu/windows/macos) green — gpui compile happens only under `--features gpui-ui`; add a CI job building the gallery example so the UI path is covered
5. `Cargo.lock` committed with the resolved gpui/gpui-component SHAs recorded in the workspace manifest comment

## Risks & mitigations

- **Two fast-moving upstreams (zed gpui + gpui-component), both pre-1.0** → lock-pin both, always bump together, wrap-layer contains API drift; budget for breakage per bump.
- **gpui-component may lack a primitive we expected to wrap** → build it custom (SeekBar/Marquee/Artwork already custom; worst case adds Button/Input too).
- **Windows is gpui's newest platform leg** → gallery must run on this machine from Task 4 onward; don't defer Windows testing to the end.
- **Compile times with gpui are heavy** → `gpui-ui` feature keeps engine/CLI builds lean; gallery example is a separate compilation unit.

## Out of scope (later phases)

Organisms (U2: NowPlayingBar, TrackList, Sidebar…), app shell + MockEngine (U3), pages (U4), dynamic album-art palette, real engine integration (U5). The four Lovable `.tsx` components (`app-shell`, `now-playing-bar`, `track-list`, `album-card`) become the design reference at U2+.
