# Master Plan: Termusic Engine Adoption & Authenticated Innertube Pipeline

## 1. Project Goal
Transform RusticPlayer from a static/dummy UI into a fully functional, 100% native Rust, GPU-accelerated YouTube Music desktop player using:
1. **`termusic-playback`** (`features = ["rusty"]`: pure Rust Symphonia + CPAL + async_ringbuf) for audio decoding and playback with zero external C DLL dependencies on Windows.
2. **Authenticated Innertube Pipeline** (`cookies.json` + `SAPISIDHASH` + `WEB_REMIX` / `ANDROID_MUSIC`) to ingest real personalized home recommendations, dynamic UP NEXT radio queues, and live lyrics.
3. **Range-Buffered Seekable MediaSource** for Symphonia to stream remote audio formats without downloading entire files.
4. **Decoupled Tokio ↔ GPUI Bridge** maintaining 60fps buttery-smooth seekbar and animated ambient blur without blocking the UI frame loop.

---

## 2. Multi-Worker Delegation Architecture

| Worker | Role | Strengths / Responsibilities |
| :--- | :--- | :--- |
| **Host Agent (Flash)** | Supervisor, Orchestrator & Fast Implementer | Direct file edits, wiring, cargo builds, tests, GPUI views, supervisor verification of diffs. |
| **OpenCode Operator (`operator`)** | Autonomous Deep Implementation Worker | Heavy, self-contained implementation modules running concurrently in background via `oc_dispatch`. |
| **Oracle (`CLAUDE_OPUS_48_BEDROCK`)** | Architectural Invariant & Adversarial Review | Zero-overhead direct consultation on wire protocols, cipher transforms, and audio I/O contracts. |

---

## 3. Phased Implementation Roadmap

### Phase 1: Authenticated Innertube Client & Polymorphic Parser (`crates/sources`)
- **Phase 1A (Host)**: Cookie parser, multi-hash `SAPISIDHASH` generator (`origin: https://music.youtube.com`), and session state machine (Authenticated vs. Guest `visitorData` fallback).
- **Phase 1B (Operator)**: Resilient Innertube JSON Parser (`browse` shelves, `next` queue, `lyrics`).
- **Phase 1C (Host)**: Integration and verification against live YouTube servers via `test_cookies`.

### Phase 2: Termusic Audio Engine & Seekable HTTP MediaSource (`crates/audio` & `crates/engine`)
- **Phase 2A (Operator)**: Range-Buffered Seekable `MediaSource` for Symphonia.
- **Phase 2B (Host)**: Integrate `termusic-playback` into `crates/engine`.
- **Phase 2C (Host)**: Lazy Stream Resolver (prefetch depth 1) with client fallback ladder (`ANDROID_MUSIC` -> `TVHTML5` -> `WEB_REMIX`).

### Phase 3: Reactive Tokio ↔ GPUI Bridge (`crates/app`)
- **Phase 3A (Host)**: Bounded async channel bridge with 4Hz progress tick throttling and reliable state transitions.
- **Phase 3B (Host)**: 60fps `PlaybackTimeline` local interpolation and seekbar scrub reconciliation.

### Phase 4: Dynamic Frontend UI Population (`crates/app/src/pages.rs`)
- **Phase 4A (Host)**: Dynamic `HomePage` with live carousels ("Listen again", "Quick picks", "Forgotten favorites") and mood chips.
- **Phase 4B (Host)**: Dynamic `PlayerPage` with live UP NEXT radio queue and live lyrics.
- **Phase 4C (Host)**: Dynamic Gaussian ambient radiance adapting to the current track's album art.

### Phase 5: Verification & Hardening
- **Phase 5A (Host)**: CPAL/WASAPI device recovery (Bluetooth drop / sleep-wake).
- **Phase 5B (Host)**: End-to-end testing, visual screenshot verification, and performance profiling.
