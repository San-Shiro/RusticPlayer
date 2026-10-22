# SimpMusic — Feature & Implementation Report

**Repository URL**: [https://github.com/maxrave-dev/SimpMusic](https://github.com/maxrave-dev/SimpMusic)  
**Version Analyzed**: `1.0.4+` (Kotlin / Compose Multiplatform)  
**Local Code Location**: `reference/SimpMusic`  
**License**: GPL-3.0  

---

## 1. Executive Summary & Core Architectural Concept

SimpMusic is an open-source, cross-platform music streaming client for Android and Desktop (Windows, macOS, Linux) built with **Compose Multiplatform** and **Clean Architecture**.
- Primary music catalog is scraped directly from YouTube & YouTube Music.
- Incorporates **Spotify Canvas** (short looping background videos for tracks) and **AI-Assisted Music Recommendations** (via OpenAI / Google Gemini).
- Built around a decoupled multi-module architecture (`composeApp`, `androidApp`, `desktopApp`, `core/*`).

---

## 2. Technical Stack & Module Architecture

```
                               ┌─────────────────────────────────┐
                               │   Presentation (composeApp)     │
                               │   Jetpack Compose Multiplatform │
                               └────────────────┬────────────────┘
                                                │
                               ┌────────────────▼────────────────┐
                               │     Domain (core/domain)        │
                               │   Use Cases & Domain Models     │
                               └────────────────┬────────────────┘
                                                │
                               ┌────────────────▼────────────────┐
                               │       Data (core/data)          │
                               │  Repositories, Room, DataStore   │
                               └────────────────┬────────────────┘
                                                │
       ┌───────────────────────┬────────────────┴───────┬───────────────────────┐
       ▼                       ▼                        ▼                       ▼
┌───────────────┐     ┌─────────────────┐     ┌──────────────────┐    ┌──────────────────┐
│  core/service │     │   core/media    │     │      lastfm      │    │       cast       │
│  ytmusic      │     │  Media3 (mob)   │     │  (or lastfm-empty│    │ (or cast-empty)  │
│  spotify, AI  │     │  libmpv (desk)  │     └──────────────────┘    └──────────────────┘
└───────────────┘     └─────────────────┘
```

| Module | Purpose & Tech |
| :--- | :--- |
| **`composeApp/`** | Shared Compose UI for Android & Desktop JVM |
| **`androidApp/`** | Android application entry point & Manifest |
| **`desktopApp/`** | Desktop JVM application entry point |
| **`core/domain/`** | Business logic use cases & domain models |
| **`core/data/`** | Room DB, DataStore, Repository implementations |
| **`core/media/media3/`** | Android Media3 ExoPlayer + `CrossfadeExoPlayerAdapter` |
| **`core/media/media-jvm/`**| Desktop audio playback using **libmpv** bound via JNA |
| **`core/service/kotlinYtmusicScraper/`** | YouTube Music API scraper |
| **`core/service/spotify/`** | Spotify API, Spotify Canvas MP4 fetcher |
| **`core/service/aiService/`** | OpenAI & Google Gemini API integrations |
| **`core/service/lyricsService/`** | Synced lyrics engine (LRCLIB, BetterLyrics) |
| **`core/service/kizzy/`** | Discord Rich Presence RPC |
| **`lastfm/` vs `lastfm-empty/`** | Modular Last.fm scrobbler (FOSS stub abstraction) |
| **`cast/` vs `cast-empty/`** | Modular Google Cast integration (FOSS stub abstraction) |

---

## 3. Audio Engine & Crossfade Implementation

### 3.1 Dual Media Engine (`core/media/`)
- **Android**: Uses Android **Media3 (ExoPlayer)**. Features `CrossfadeExoPlayerAdapter` which maintains two underlying ExoPlayer instances to smoothly blend audio between the ending track and starting track (DJ-style crossfade).
- **Desktop (JVM)**: Uses **libmpv** (the native C API of mpv bound through JNA). Replaced previous VLCJ/GStreamer engines for lower latency and better gapless playback on Windows, macOS, and Linux.

### 3.2 FOSS Modular Flavors (`cast-empty`, `lastfm-empty`, `crashlytics-empty`)
SimpMusic uses Gradle module substitution (`isFullBuild`) to decouple proprietary or telemetry services:
- **Full Build**: Includes Google Cast (`media3-cast`), Sentry Crashlytics, and Last.fm API secrets.
- **FOSS Build**: Replaces those modules with zero-dependency `-empty` stubs (`cast-empty`, `lastfm-empty`) to comply with F-Droid guidelines.

---

## 4. Integrations & Feature Implementations

### 4.1 YouTube Music Scraper (`core/service/kotlinYtmusicScraper`)
- Implements custom HTTP scrapers for YouTube Music internal endpoints.
- Supports browsing home feeds, chart rankings, artist discographies, related radio seeds, and auto-queue generation.

### 4.2 Spotify Canvas Integration (`core/service/spotify`)
- Queries Spotify API for track metadata matches and extracts Spotify Canvas `.mp4` video stream URLs.
- Plays looping canvas videos in the Compose player view underneath or replacing static album cover art.

### 4.3 AI Music Recommendations (`core/service/aiService`)
- Integrates with **OpenAI API** and **Google Gemini API**.
- Users can type natural language prompts (e.g., *"Upbeat 80s synthwave for late night coding"*).
- The AI service converts the prompt into structured search queries and seed tracks, fetching matching songs from YouTube Music to generate a custom playlist.

### 4.4 Synced Lyrics Service (`core/service/lyricsService`)
- Multi-provider fallback chain: **LRCLIB** → **BetterLyrics** → **SimpMusic Lyrics API** → **Spotify Lyrics**.
- Delivers real-time synced lyrics with word-level or line-level timestamps.

### 4.5 Last.fm Scrobbler (`lastfm/`)
- Talks to `ws.audioscrobbler.com/2.0/` over form-urlencoded POST requests signed with Okio MD5 (`api_sig`).
- Uses Web OAuth flow (`last.fm/api/auth/?api_key=X`) with deep-link callback (`wordbyword://lastfm-auth`) to prevent password exposure.

### 4.6 SponsorBlock & ReturnYouTubeDislike
- Queries SponsorBlock to auto-skip intros, non-music sections, and sponsor ads.
- Queries ReturnYouTubeDislike to display track rating/dislike statistics.

---

## 5. UI Architecture & Design Details

- **UI Framework**: Jetpack Compose Multiplatform adhering to Material Design 3 guidelines.
- **Player Screen**: Displays video canvas loops, real-time synced lyrics scrolling, dynamic album art color palettes, and transport controls.
- **State Flow**: Unidirectional Data Flow (UDF) using `StateFlow` and Compose `State<T>`.

---

## 6. Key Takeaways for Rustic Player

1. **Crossfade Adapter**: The `CrossfadeExoPlayerAdapter` pattern (or dual decoder buffers) provides an exceptional listening experience for gapless and seamless track transitions.
2. **AI Recommendation Engine**: Integrating LLM prompts for natural language playlist generation is a compelling modern feature.
3. **Modular Dependency Separation**: Using stub modules (`-empty`) to keep clean separation between core engine logic and third-party integrations.
