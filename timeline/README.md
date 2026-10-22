# RusticPlayer — Master Development Timeline & Phase Roadmap

> **Target Quality Gate**: 100% Pure Rust, Zero-C-DLLs, native GPUI rendering, <50MB RAM footprint, ~0% idle CPU, and <0.8% playback CPU.

---

## Phasing Strategy & Order of Execution

```
[Phase 1: Queue System & UI Perfection] ──▶ [Phase 2: Homepage & Settings] ──▶ [Phase 3: Local DB, Offline Sync & History]
          (CURRENT FOCUS)                               (UPCOMING)                                (DEFERRED)
```

1. **Phase 1: Multi-Lane Queue System & UI Perfection (CURRENT PRIORITY)**
   - Solidify the **3-Lane Queue Model** (User Priority Queue + Context/Album Queue + Automix Radio Stream) inside `QueueManager`.
   - Eliminate the primitive flat `self.state.queue` in `crates/engine` and make `QueueManager` the authoritative playback driver.
   - Implement the automated YouTube Music recommendation engine (`RDAMVM` automix, prefetch depth 1, continuation streaming).
   - Build a pixel-perfect, virtualized, 3-lane Queue & Up Next panel in GPUI.

2. **Phase 2: Actual Homepage Design & Settings**
   - High-fidelity homepage carousels ("Listen again", "Quick picks", "Mixed for you").
   - Functional mood & genre chips filter system.
   - Comprehensive Settings panel: WASAPI audio output selector, cookie/auth manager, Discord RPC toggle, cache limits.

3. **Phase 3: Local Database, Offline Sync & Google History**
   - Full SQLite schema with WAL mode & FTS5 instant local search.
   - Offline download manager integration with playback resolution: play local files directly if downloaded.
   - Google Watchtime & History registration beacon (`videostatsPlaybackUrl` + CPN) so played songs appear in official YouTube Music history.

---

## Active Phase Breakdown Files

- [`phase-1-queue-system.md`](file:///f:/VSCodium/Github/RusticPlayer/timeline/phase-1-queue-system.md): **Immediate Sprint Checklist & Architectural Spec**
- [`phase-2-homepage-and-settings.md`](file:///f:/VSCodium/Github/RusticPlayer/timeline/phase-2-homepage-and-settings.md): **UI Design, Browsing & Configuration Spec**
- [`phase-3-local-db-and-offline-sync.md`](file:///f:/VSCodium/Github/RusticPlayer/timeline/phase-3-local-db-and-offline-sync.md): **Storage, Offline Playback & History Spec**
