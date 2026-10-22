# UI Framework Decision — Rust Music Player (Linux-first, Windows + macOS)

**Constraints established:** <150MB RAM, very low idle, fast/responsive, blur + animation, Linux is the priority target, minimise development effort, possible web version later.

---

## TL;DR

**Build it in GPUI + gpui-component.**

The reason this is now the answer and wasn't three days ago: `gpui-component` is far more mature than a typical third-party widget set. It has 11.6k stars, 60+ components, an Apache-2.0 licence, and — critically — **virtualized Table and List built in**. That was the single biggest item on your "rebuild it yourself" list, and it's gone.

**Fallback if you want maximum community safety:** Iced.
**Fallback if you want maximum shipping speed and accept Linux jank:** Tauri + Solid.js.

---

## 1. The pre-made assets question (this is the deciding factor)

### GPUI — `gpui-component` (longbridge)

| | |
|---|---|
| Stars / forks | 11.6k / 642 |
| Licence | Apache-2.0 |
| Latest release | v0.5.1 (Feb 2026), ~1,875 commits |
| Design language | shadcn/ui-inspired, some Reui |
| Icons | Lucide (SVGs added by you, not bundled) |

**What ships in the box:**

- 60+ cross-platform components
- **Virtualized Table and List** (rows *and* columns) — the big one
- Dock layout: panels, split views, tabs, resizing, freeform tiles
- Built-in `Theme` / `ThemeColor` system with multiple prebuilt themes
- Size variants (`xs`/`sm`/`md`/`lg`) across components
- Charts
- Markdown + basic HTML rendering
- A code editor with LSP and Tree-sitter highlighting (irrelevant to you, but a signal of how far it's been pushed)
- A running gallery app (`cargo run`) to browse everything
- A WASM gallery target — so the web path has at least been walked

**Proof it's real:** Longbridge Pro, a shipping commercial desktop trading app, is built on it. Not a demo.

**Caveat:** the framework comparison table in their README is written by the gpui-component authors and is self-serving. Treat their "Iced: No" entries as directional, not gospel.

### Iced — the ecosystem

| | |
|---|---|
| Core stars | ~31,000 |
| Licence | MIT |
| Status | Very active (commits through Jul 2026), v0.14 |

- **`iced_aw` 0.14.1** — officially adopted into the iced-rs org. MIT, ~26K SLoC, 414k downloads. Feature-gated so you only compile what you use: badges, cards, colour picker, date picker, number input, sidebar (incl. flush column/row), menus.
- **`iced_audio`** — officially adopted too. Knobs, faders, audio-specific widgets. Directly relevant to a music player; nothing equivalent exists for GPUI.
- **`awesome-iced`** — curated list of community widgets, integrations, example projects.
- **Official book** + docs + active Discord with regular coordination meetings.
- **COSMIC desktop** (System76) is built on it — real institutional backing.

**The gap:** no virtualized list/table in the official ecosystem. You write that. No blur primitive. No animation primitive.

### Verdict on assets

GPUI's component library is *deeper per-component* and covers the two things that cost you the most time (virtualization, polished modern styling). Iced's ecosystem is *broader and better governed* — official adoption, a book, a bigger Discord — plus `iced_audio` is a genuinely nice fit for your domain.

For a music player specifically, GPUI-component's virtualized list is worth more than iced_audio's knobs.

---

## 2. Full comparison

| | **GPUI + gpui-component** | **Iced** | **Tauri + Solid** | **Flutter** |
|---|---|---|---|---|
| **Idle RAM** | 50–90MB | 50–90MB | 90–180MB (Linux worst) | 120–200MB |
| **Startup** | ~100ms | ~100ms | 200–500ms | 400–800ms |
| **Binary** | ~12MB | ~11MB | 5–15MB | 40–60MB |
| **Linux quality** | Good (self-drawn, wgpu) | Good (self-drawn, wgpu) | **Weakest link — WebKitGTK** | Good (Impeller) |
| **Blur** | Built in | You write a `Shader` | CSS `backdrop-filter`, unreliable on Linux | Built in |
| **Animation** | Built in | You hand-roll | CSS | Built in |
| **Virtual lists** | Built in | You write it | TanStack Virtual (free) | Built in |
| **Drag & drop** | You write it | You write it | dnd-kit (free) | Partial |
| **API stability** | Pre-1.0, breaking changes | Stable-ish | Stable | Stable |
| **Community** | Growing, thin docs | **Best of the Rust options** | Huge (web ecosystem) | Huge |
| **Web port later** | WASM (heavy) | WASM (heavy) | **Trivial — reuse frontend** | Poor (canvas) |
| **Time to shell** | 1–2 weeks | 2–3 weeks | 3–5 days | ~1 week |
| **Time to polished** | 4–6 weeks | 8–10 weeks | 3–4 weeks | 4–6 weeks |

---

## 3. Pros and cons, honestly

### GPUI + gpui-component ✅ recommended

**Pros**
- Blur, shadows, spring animation, rounded clipping — all built, all proven in Zed at 120fps
- Virtualized list/table solves your worst rebuild item
- Modern shadcn-style default look; you're not fighting toward "not ugly"
- Hybrid immediate/retained: write freely, only dirty regions repaint → near-zero idle
- Taffy layout = flexbox; if you know CSS flex you know it
- Two real shipping apps prove it (Zed, Longbridge Pro)

**Cons**
- **Pre-1.0. Breaking changes between revs.** Pin a git SHA; budget a day per bump.
- Docs are thin — the real reference is Zed's source and the gallery app
- You depend on *two* fast-moving upstreams (Zed's gpui + longbridge's components)
- Windows is the newest platform leg, least battle-tested
- Smaller community; fewer answers when stuck
- No audio-specific widgets

### Iced

**Pros**
- Largest, best-governed Rust GUI community; official book; responsive Discord
- Stable API — a two-year-old answer still applies
- `iced_audio` gives you faders/knobs free
- System76 dependency = long-term maintenance confidence
- Same performance envelope as GPUI

**Cons**
- Blur is a custom `Shader` primitive: 3–5 days
- Animation is hand-rolled off frame subscriptions: 1–2 weeks
- Virtualized list: ~1 week
- Default look is plain — real design work needed to not look like a utility
- Roughly **4 extra weeks** of infrastructure vs GPUI

### Tauri + Solid.js

**Pros**
- Fastest to ship by a wide margin — entire web ecosystem available
- Blur, animation, virtualization, drag-drop all solved libraries
- Clean Rust IPC (no `flutter_rust_bridge` pain)
- **Best web-port story: swap IPC for HTTP, reuse the frontend, ~1–3 weeks**

**Cons**
- **WebKitGTK on Linux** — slowest engine, flaky GPU accel, `backdrop-filter` unreliable, version varies by distro. Your priority platform is its worst.
- Highest RAM of the Rust options
- Excellent on Windows/macOS — the platforms you said you care least about

### Flutter (what you have)

**Pros**
- Consistent rendering everywhere including Linux (Impeller draws its own pixels — no WebKit lottery)
- Everything built in
- Huge ecosystem

**Cons**
- The RAM number you objected to
- `flutter_rust_bridge` friction on complex types (your original complaint)
- Slow startup, large binary
- Poor web story (canvas, bad a11y/SEO)

---

## 4. Difficulty by task — GPUI vs Iced

| Task | GPUI | Iced |
|---|---|---|
| Responsive layout (sidebar collapse, grid reflow) | Easy — flexbox | Easy — `Length::Fill` + width match |
| Theming | Free (built-in themes) | Easy |
| Track list, 10k items | **Free** (virtualized List) | ~1 week |
| Album art pipeline (decode off-thread, LRU, downscale) | 3–4 days | 3–4 days |
| Backdrop blur | **Free** | 3–5 days |
| Spring/tween animation | **Free** | 1–2 weeks |
| Context menus | Free | 3–4 days |
| Custom seek bar / waveform | 2–3 days | 2–3 days (canvas) |
| Drag-drop playlist reorder | 2 days (scoped) – 1 week (full) | Same |
| Marquee text | 1 day | 2 days |

**Rough totals to polished:** GPUI ~4–6 weeks · Iced ~8–10 weeks.

---

## 5. Architecture — do this regardless of choice

Three crates. This is the one decision that's genuinely expensive to reverse.

```
core/       pure logic — playlist, queue, state machines, metadata
            NO std::fs, NO Instant, NO tokio::net. Pass time in as a parameter.
            → compiles to wasm for free later

platform/   traits: AudioBackend, Storage, HttpClient, MediaKeys
            native impls now; web/other impls later

app/        the frontend — swappable
```

Plus:
- **All IPC/state access behind one module.** Never scatter calls through views.
- **Native shims only for system touchpoints:** MPRIS (Linux), SMTC (Windows), MPNowPlayingInfoCenter (macOS), media keys, tray. A few hundred lines each behind one Rust trait. Do *not* write native UI.

If `core` stays pure, swapping frontends later is a frontend rewrite, not a project restart.

---

## 6. Performance rules (apply to any choice)

- Never render a frame when nothing is dirty
- Progress bar ticks at **1Hz**, not 60 — only 60fps while actively dragging
- Cache blurred textures; invalidate on source change, not per frame
- Stop rendering when occluded/minimized; 30fps when unfocused
- Render thread: zero allocation, zero locking, zero I/O. Use `arc-swap` for shared state
- Audio thread: real-time priority, lock-free ring buffer, never allocates, shares no lock
- Decode + downscale images off-thread, upload via staging buffer, disk-cache thumbnails at render size
- Cache measured text; re-measure only on string/width change
- `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`
- Wire up **tracy** from day one. Track p99 frame time and input-to-photon, not averages

---

## 7. Decision rules — pick by what you actually fear

| If your biggest fear is... | Choose |
|---|---|
| Spending months on UI plumbing | **GPUI + gpui-component** |
| Being stuck alone with no answers | **Iced** |
| Never shipping at all | **Tauri + Solid** |
| Linux users having a bad experience | **GPUI or Iced** (not Tauri) |
| Needing a real web app in a year | **Tauri** |

---

## 8. Next action

Don't decide from this document. **Spend two days building the same thing in GPUI and in Iced:** a sidebar, a track list, a now-playing bar.

You'll know within a week which one you want to live in — and that's worth more than any further analysis. You've been iterating on this decision for a while; a spike converts opinion into evidence.

Start `core/` first, either way. It's valid work under every option on this list.
