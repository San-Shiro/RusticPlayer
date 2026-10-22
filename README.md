# RusticPlayer 🎵🦀

**RusticPlayer** is a high-performance, native music player and UI engine testbed written in 100% Rust—designed to push **iGPU-accelerated rendering** and **full-stack Rust architecture** across all devices.

![RusticPlayer Preview](screenshot.png)

---

## 🌟 Vision & Philosophy

### 1. Building a Powerful Rust UI Engine with the iGPU at its Core
Modern desktop and cross-device software is weighed down by bloated web wrappers. **RusticPlayer** takes the opposite path: building and refining a powerful, declarative UI engine written entirely in Rust (built atop **GPUI**) that treats the **integrated GPU (iGPU)** as its core execution engine. By offloading layout primitives, shaders, backdrop blurs, and animations directly to the iGPU, we achieve rock-solid 60–120+ FPS frame pacing with negligible CPU overhead and minimal memory footprint—even on lightweight laptops and low-power hardware.

### 2. Advancing Rust as a True Full-Stack Language for All Devices
Our mission is to push Rust forward as a complete, unified full-stack ecosystem capable of targeting every device class without compromise:
- **Systems & Hardware Layer**: Lock-free ring buffers, direct OS audio hal (`CPAL`), and hardware DAC synchronization.
- **Data & Networking Layer**: Asynchronous streaming, sliding-window HTTP range caches, Innertube protocol extraction, and embedded SQLite persistence.
- **Presentation & UI Layer**: A composable, GPU-driven component architecture (`crates/ui-kit`) with reactive state synchronization, dynamic color extraction, and zero-cost abstractions.

### 3. Starting with a Music Player, Expanding Ahead
Just as **Zed** (and **Zen**) proved that the best way to forge a next-generation UI engine and design philosophy is to build a demanding flagship product around it, **RusticPlayer** is our foundation. A music player demands both hard real-time audio guarantees and fluid visual responsiveness. We are starting here—perfecting the rendering primitives, theming engine, and full-stack ergonomics—and will steadily extend these foundations into a broader suite of cross-device native applications.

---

## ⚡ Highlights & Features

- **🖥️ iGPU-First Desktop Engine**: Native hardware-accelerated rendering with sub-millisecond frame times, dynamic artwork palette shaders, and zero Electron/WebView overhead.
- **🔊 Bit-Exact Lock-Free Audio Pipeline**:
  - Hard real-time audio callback powered by **CPAL** and `rtrb` ring buffers (allocation-free, lock-free, zero-glitch).
  - Multi-format decoding via **Symphonia** supporting MP3, AAC, FLAC, Ogg, and Opus.
  - Studio-grade resampling with **Rubato**.
  - Hardware DAC-synchronized **Master Clock** for sample-accurate timeline seeking.
- **🎯 3-Lane Smart Queue Architecture**:
  - **Lane 1 (User Queue)**: Immediate priority queue for tracks added via *Play Next* or *Add to Queue*.
  - **Lane 2 (Context Queue)**: Active album, playlist, or discography context.
  - **Lane 3 (Radio & Suggestions)**: Dynamic automix and endless autoplay recommendations.
- **🌐 YouTube Music & Streaming Integration**:
  - High-res audio stream extraction using an Innertube multi-client fallback ladder (`ANDROID_MUSIC` → `VISIONOS` → `WEB_REMIX` → `TVHTML5`).
  - Seekable ranged HTTP streaming with a sliding chunk LRU cache.
  - Seamless authenticated or guest browsing, search, and dynamic playlists.
- **🎨 Composable UI Kit (`crates/ui-kit`)**:
  - Built-from-scratch design system with reusable atomic components, theme tokens, and smooth viewport choreography.
  - Fluid Cyberpunk and custom theme presets with iGPU-accelerated backdrop blur.

---

## 🏗️ Workspace Architecture

RusticPlayer is structured as a modular full-stack Rust workspace, separating the UI engine kit, application shell, real-time audio core, and network providers:

```
crates/
├── ui-kit/       # Core iGPU design system: atomic components, tokens, shaders, icons
├── app/          # GPUI desktop frontend, view routing, and reactive state bridge
├── audio/        # CPAL output, Symphonia decoder, Rubato resampler, HTTP source
├── engine/       # Background orchestrator loop, SQLite state, command dispatch
├── rustic_core/  # 3-lane queue state machine, domain models, cross-device contracts
├── sources/      # YouTube Music Innertube client, cipher engine, Spotify provider
├── platform/     # OS media controls (SMTC / MPRIS), tray, native notifications
├── jam/          # Synchronized collaborative session protocols (WebRTC / LAN)
├── services/     # Scrobblers (Last.fm, ListenBrainz), Discord RPC
└── cli/          # Headless test harnesses and engine diagnostics
```

---

## 🚀 Building & Running

### Prerequisites

- **Rust toolchain** (1.80+ or latest stable): [rustup.rs](https://rustup.rs/)
- **C/C++ Build Tools**: MSVC (on Windows) or GCC/Clang with ALSA/Pulse & Vulkan/DirectX headers (on Linux/Windows)

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

## 🗺️ Roadmap Ahead

- [x] Lock-free audio engine & DAC-synced master clock
- [x] 3-Lane smart queue state machine & Innertube streaming pipeline
- [x] Core `ui-kit` component primitives & dynamic theme engine on GPUI
- [ ] Custom WGSL/iGPU visualizers & real-time audio spectrum shaders
- [ ] Offline encrypted cache & full local library synchronization
- [ ] Multi-device Jam sessions & portable UI target expansion

---

## 📄 License

This project is licensed under the Apache 2.0 / MIT License.
