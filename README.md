# RusticPlayer 🎵🦀

**RusticPlayer** is a modern, high-performance desktop music player written in Rust, powered by **GPUI** (Zed's GPU-accelerated UI framework) and a dedicated lock-free audio engine.

![RusticPlayer Preview](screenshot.png)

---

## Highlights & Features

- **⚡ Blazing Fast GPUI Desktop Interface**: Native 60+ FPS desktop interface with minimal CPU and memory footprints.
- **🔊 Bit-Exact Lock-Free Audio Pipeline**:
  - Hard real-time audio callback powered by **CPAL** and `rtrb` ring buffers (allocation-free, lock-free, zero-glitch).
  - Format decoding via **Symphonia** supporting MP3, AAC, FLAC, Ogg, and Opus.
  - High-fidelity resampling with **Rubato**.
  - Hardware DAC-synchronized **Master Clock** for glitch-free timeline seeking.
- **🎯 3-Lane Smart Queue Architecture**:
  - **Lane 1 (User Queue)**: Immediate priority queue for tracks added via *Play Next* or *Add to Queue*.
  - **Lane 2 (Context Queue)**: Active album, playlist, or discography context.
  - **Lane 3 (Radio & Suggestions)**: Dynamic automix and endless autoplay recommendations.
- **🌐 YouTube Music & Streaming Integration**:
  - High-res audio stream extraction using an Innertube multi-client fallback ladder (`ANDROID_MUSIC` → `VISIONOS` → `WEB_REMIX` → `TVHTML5`).
  - Seekable ranged HTTP streaming with a sliding chunk LRU cache.
  - Seamless authenticated or guest browsing, search, and dynamic playlists.
- **🎨 Modern Design & Theming**:
  - Fluid Cyberpunk and custom theme presets with subtle backdrop blur.
  - Dynamic palette extraction from album artwork.
  - Responsive layout with expandable sidebar, queue panel, and lyrics views.

---

## Workspace Architecture

RusticPlayer follows a decoupled modular crate design:

```
crates/
├── app/          # GPUI desktop frontend, view routing, and state bridge
├── audio/        # CPAL output, Symphonia decoder, Rubato resampler, HTTP source
├── engine/       # Background orchestrator loop, SQLite state, command dispatch
├── rustic_core/  # 3-lane queue state machine, API models, contracts
├── sources/      # YouTube Music Innertube client, cipher engine, Spotify provider
├── ui-kit/       # Reusable GPUI atomic design components, tokens, icons
├── platform/     # OS media controls (SMTC / MPRIS), tray, notifications
├── jam/          # Synchronized collaborative session protocols (WebRTC / LAN)
├── services/     # Scrobblers (Last.fm, ListenBrainz), Discord RPC
└── cli/          # Headless test harnesses and diagnostics
```

---

## Building & Running

### Prerequisites

- **Rust toolchain** (1.80+ or latest stable): [rustup.rs](https://rustup.rs/)
- **C/C++ Build Tools**: MSVC (on Windows) or GCC/Clang with ALSA/Pulse development headers (on Linux)

### Run the Desktop App

```bash
# Clone the repository
git clone https://github.com/San-Shiro/RusticPlayer.git
cd RusticPlayer

# Run the desktop application
cargo run -p app --release
```

Or on Windows:
```cmd
run_app.bat
```

### Running Test Suite

```bash
cargo test --workspace
```

---

## License

This project is licensed under the Apache 2.0 / MIT License.
