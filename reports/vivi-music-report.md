# ViVi Music — Feature & Implementation Report

**Repository URL**: [https://github.com/vivizzz007/vivi-music](https://github.com/vivizzz007/vivi-music)  
**Version Analyzed**: Latest main (`33c82f9`) (Kotlin / Jetpack Compose)  
**Local Code Location**: `reference/vivi-music`  
**License**: GPL-3.0  

---

## 1. Executive Summary & Core Architectural Concept

ViVi Music is a feature-rich, high-performance Android music streaming application built with **Kotlin** and **Jetpack Compose**.
- Combines multiple streaming sources: **YouTube Music (Innertube)**, **JioSaavn** (regional & Indian catalog), and **Spotify API** (metadata & canvas).
- Includes advanced audio capabilities like **Shazam-style Audio Song Recognition**, **Apple Music-style Animated Canvases**, **Karaoke Word-by-Word Synced Lyrics**, and **Offline Audio Caching**.
- Built around an ultra-modular Gradle multi-module codebase.

---

## 2. Technical Stack & Multi-Module Architecture

```
                                 ┌────────────────────────┐
                                 │     app / NEW-UI       │
                                 │    Jetpack Compose     │
                                 └───────────┬────────────┘
                                             │
      ┌─────────────┬─────────────┬──────────┴──┬─────────────┬─────────────┬─────────────┐
      ▼             ▼             ▼             ▼             ▼             ▼             ▼
┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐
│ innertube │ │ jiosaavn  │ │  spotify  │ │ shazamkit │ │  canvas   │ │ lyrics    │ │  lastfm   │
│ (YT Music)│ │ (JioSaavn)│ │ (Canvas)  │ │ (Audio ID)│ │ (Visuals) │ │ Provider  │ │ & kizzy   │
└───────────┘ └───────────┘ └───────────┘ └───────────┘ └───────────┘ └───────────┘ └───────────┘
```

| Module | Feature / Purpose | Implementation Details |
| :--- | :--- | :--- |
| **`app/`** | Android Application & Main UI | Jetpack Compose, Material 3, Navigation, ViewModels |
| **`innertube/`** | YouTube Music Client | Search, playlists, artist discography, streaming URL resolution |
| **`jiosaavn/`** | JioSaavn API Integration | Streaming Indian & regional tracks, albums, charts directly |
| **`spotify/`** | Spotify API Integration | Metadata lookups, playlist import, Spotify Canvas video loops |
| **`shazamkit/`** | Audio Song Recognition | Takes microphone audio sample and identifies unknown songs |
| **`lyricsProvider/`** | Multi-Source Lyrics Engine | LRCLIB, Musixmatch, Genius, Spotify, K-Lyrics (word-by-word) |
| **`applecanvas/`** | Apple Music Visualizers | Apple Music-style video background loops |
| **`canvas/` / `vivimusiccanvas/`** | Video Canvas Engine | Render engine for track canvas video backdrops |
| **`artistvideo/`** | Artist Video Streams | Background artist video playback |
| **`lastfm/`** | Last.fm Scrobbler | Audio scrobbling integration |
| **`kizzy/`** | Discord RPC | Discord Rich Presence status broadcast |

---

## 3. Audio Engine & Caching Pipeline

### 3.1 Media3 / ExoPlayer Engine (`app/src/main/java/com/vivi/music/service/`)
- Powered by Android **Media3 (ExoPlayer)**.
- Implements `CacheDataSource` with `SimpleCache` to store streamed audio chunks locally on disk. If a user replays a song, it serves directly from local disk cache without re-downloading bytes.
- Supports dynamic pitch shifting, tempo adjustments, custom parametric equalizer presets, and ReplayGain loudness normalization.

---

## 4. Integrations & Deep-Dive Feature Breakdown

### 4.1 Innertube (YouTube Music Scraper) (`innertube/`)
- Parses YouTube Music API responses for song streams, video streams, artist radios, and related tracks.
- Resolves high-bitrate Opus and AAC streams with fallback endpoints.

### 4.2 JioSaavn Source Integration (`jiosaavn/`)
- Connects to JioSaavn API (`jiosaavn.md`).
- Searches tracks, albums, playlists, and top charts across multiple languages (Hindi, English, Punjabi, Tamil, Telugu, etc.).
- Direct audio stream URL extraction without relying on YouTube matching for regional titles.

### 4.3 ShazamKit Audio Recognition (`shazamkit/`)
- Listens to audio via microphone input.
- Generates audio signature fingerprints and queries song recognition servers to identify ambiently playing music.
- Automatically opens the recognized song inside ViVi Music for immediate playback.

### 4.4 Karaoke Word-by-Word Lyrics (`lyricsProvider/`)
- Aggregates synced lyrics from **Lrclib**, **Musixmatch**, **Genius**, **Spotify**, and **K-Lyrics**.
- Supports **Word-by-Word Karaoke highlighting**: text highlights syllable-by-syllable in sync with the audio track.

### 4.5 Canvas & Visualizer Suite (`applecanvas/`, `canvas/`, `vivimusiccanvas/`)
- Renders Apple Music-style animated background canvases underneath the main player sheet.
- Plays track-specific canvas video loops fetched via the `spotify` module.

---

## 5. UI Architecture & Design Details

- **Material You Design**: Dynamic color palette extracted from the active album cover art using Android's `Palette` API.
- **Fluid Micro-Animations**: Smooth bottom sheet expansion for the player bar, animated audio wave bars, and fluid list scrolling.
- **Privacy-First**: No background telemetry, all databases (Room) are 100% local.

---

## 6. Key Takeaways for Rustic Player

1. **JioSaavn Integration**: Adding JioSaavn or regional source providers expands catalog coverage for regional/Indian music where YouTube tags may be inconsistent.
2. **Audio Recognition (ShazamKit)**: An innovative feature allowing users to identify songs from ambient audio.
3. **Word-by-Word Synced Lyrics**: Enhances lyrics display from simple line-by-line LRC to immersive karaoke-style word highlighting.
