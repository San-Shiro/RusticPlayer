# Phase 2: Actual Homepage Design, Exploration & Settings

> **Sprint Goal**: Transform the home feed into a rich, personalized YouTube Music experience with dynamic carousels, responsive mood filters, and build a native Settings panel for audio device configuration, session management, and integrations.

---

## 1. Homepage Design & Exploration Experience

### Step 2.1: Dynamic Personalized Shelves (`crates/app/src/pages.rs`)
- [ ] **T2.1.1 — Authentic Innertube Carousel Rendering**:
  - Implement full parsing and presentation for standard YouTube Music shelf variants:
    - `"Listen again"` (Recently played artist/track circles).
    - `"Quick picks"` (Instant song grid with play on hover).
    - `"Mixed for you"` (My Supermix, Discover Mix, New Release Mix).
    - `"Similar to ..."` (Dynamic artist radios).
    - `"Trending community playlists"`.
- [ ] **T2.1.2 — Carousel Navigation & Multi-Row Cards**:
  - Add smooth horizontal carousel scrolling with left/right scroll buttons.
  - Implement 2-row and 4-row compact song carousels (matching YouTube Music's desktop layout).
- [ ] **T2.1.3 — Interactive Mood & Genre Chips**:
  - Hook up category chips: **Relax**, **Energize**, **Workout**, **Focus**, **Commute**, **Podcasts**.
  - Tapping a chip dynamically queries Innertube's `params` token to replace home shelves with mood-filtered carousels.
- [ ] **T2.1.4 — Skeleton Loaders & Error Resilience**:
  - Add shimmering skeleton cards while the home feed or mood tab is loading over the network.
  - Add an in-place retry button if an authenticated request fails due to temporary network drop.

---

## 2. Desktop Settings & Configuration

### Step 2.2: Settings Panel Architecture (`crates/app/src/pages.rs`)
- [ ] **T2.2.1 — Audio Output Device Selector**:
  - Query CPAL for all host audio output devices (`cpal::traits::HostTrait::output_devices`).
  - Dropdown menu allowing hot-switching the active audio device (Speakers, Headphones, USB DAC, Virtual Cable) without restarting the app.
  - Real-time buffer latency and sample rate indicator (44.1kHz vs 48kHz, 24-bit).
- [ ] **T2.2.2 — Account & Authentication Management**:
  - Clear visual indicator of login state: Google Account name, profile photo, and session validity.
  - One-click "Reload cookies.json" and "Sign Out" actions.
  - Safe in-app cookie paste/import dialog with format validation.
- [ ] **T2.2.3 — Desktop Integrations (Discord RPC & System Controls)**:
  - Toggle switch for Discord Rich Presence ("Show what I'm listening to on Discord").
  - Toggle switch for System Media Transport Controls (Windows SMTC / Linux MPRIS).
  - Toggle switch for hardware media keys capturing.
- [ ] **T2.2.4 — Appearance & Performance Tuning**:
  - Dynamic Album Radiance (ambient Gaussian blur backdrop) toggle & intensity slider.
  - Disk cache size limit input (default 500 MB) and "Clear Cache" button.
  - Theme preset selector (Rustic Dark, Slate Midnight, Pitch Black OLED, Light).

---

## 3. Verification & Acceptance Criteria

1. **Pixel-Perfect Scaling**: Home page gracefully resizes across 800px, 1200px, and 1600px+ monitors with adaptive carousel card sizing.
2. **Instant Hot-Switching**: Changing the audio output device in Settings seamlessly transfers audio playback with zero stutter or crash.
3. **Persisted Configuration**: User preferences (active device, theme, cache limits, Discord RPC toggle) are saved to SQLite and survive restarts.
