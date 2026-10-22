# Namida — Feature & Implementation Report

**Repository URL**: [https://github.com/namidaco/namida](https://github.com/namidaco/namida)  
**Version Analyzed**: `6.4.1-beta` (Dart / Flutter)  
**Local Code Location**: `reference/namida`  
**License**: GPL-3.0  

---

## 1. Executive Summary & Core Architectural Concept

Namida is a feature-packed, visual-first music and video player built with Flutter. Unlike single-source players, Namida is engineered as a **Universal Audio & Video Hub**:
- Seamlessly merges **Local Offline Music**, **Subsonic / Navidrome Servers**, **WebDAV Cloud Storage**, **SMB (Windows Network Shares / NAS)**, and **YouTube** into a unified library and player interface.
- Includes a native C TagLib wrapper (`flutter_taglib`) for direct tag editing (ID3v2, FLAC comments).
- Features visual waveform rendering extracted from audio files for high-precision waveform seekbars.

---

## 2. Technical Stack & Dependencies

| Component | Library / Framework | Path / Usage |
| :--- | :--- | :--- |
| **UI Framework** | Flutter 3.44+ / Dart 3.12+ | `lib/` |
| **Audio Engines** | `just_audio` (Android/ExoPlayer), `media_kit` (Desktop/libmpv) | `lib/controller/player_controller.dart` |
| **Video Engine** | `media_kit_video`, `ffmpeg_kit_flutter` | `lib/controller/video_controller.dart` |
| **Native Tagging** | `flutter_taglib` (TagLib C bindings) | `lib/controller/tagger_controller.dart` |
| **Subsonic / Navidrome** | `opensubsonic_api` | `lib/controller/music_web_server/` |
| **WebDAV Cloud** | `webdav_client` | `lib/controller/music_web_server/` |
| **SMB (NAS)** | `smb_connect` | `lib/controller/music_web_server/` |
| **YouTube Extractor** | `youtipie` | `lib/youtube/` |
| **Waveform Extractor** | `waveform_extractor` | `lib/controller/waveform_controller.dart` |
| **Database & Cache** | `namico_db_wrapper`, `playlist_manager`, `history_manager` | `lib/controller/` |
| **UI Effects** | `dynamic_color`, `flutter_animate`, `animated_background`, `flutter_tilt` | `lib/ui/` |

---

## 3. Audio & Video Engine Architecture

### 3.1 Dual-Engine Abstraction Layer (`lib/controller/player_controller.dart`)
Namida implements a hybrid audio pipeline depending on the platform and media type:
1. **Mobile (Android/iOS)**: Uses `just_audio` (backed by Android ExoPlayer). Provides low-power background audio, system notification integration, and native Android audio focus.
2. **Desktop (Windows/Linux/macOS) & Video**: Uses `media_kit` (backed by `libmpv`). Handles video decoding, complex audio stream switching, and desktop hardware acceleration.

```
                    ┌─────────────────────────┐
                    │    PlayerController     │
                    └────────────┬────────────┘
                                 │
           ┌─────────────────────┴─────────────────────┐
           ▼                                           ▼
┌────────────────────┐                       ┌────────────────────┐
│     just_audio     │                       │     media_kit      │
│ (Android ExoPlayer)│                       │ (Desktop / libmpv) │
└────────────────────┘                       └────────────────────┘
```

### 3.2 Audio Waveform Extraction (`lib/controller/waveform_controller.dart`)
- Uses `waveform_extractor` to decode audio samples off-thread and extract peak amplitudes.
- Stores normalized waveform arrays in local storage cache (`storage_cache_manager.dart`).
- Renders an interactive, scrubbable waveform visualizer in the main player view.

---

## 4. Multi-Source Integrations & Implementation

### 4.1 Subsonic / Navidrome Integration (`lib/controller/music_web_server/`)
- Uses `opensubsonic_api` to authenticate via MD5 salt/token (`rest/ping`, `rest/search3`, `rest/getSong`).
- Streams audio directly over HTTP while caching track metadata into local history & search indexers.

### 4.2 WebDAV Cloud Integration
- Connects to Nextcloud, ownCloud, or generic WebDAV endpoints using `webdav_client`.
- Browses remote file directories, streams audio files directly, and extracts metadata over HTTP range headers.

### 4.3 SMB / Samba (NAS) Integration
- Connects to local network storage over the SMB protocol via `smb_connect`.
- Enables playing lossless FLAC/WAV files stored on home NAS servers without uploading to third-party clouds.

### 4.4 YouTube & Video Integration (`lib/youtube/`, `youtipie`)
- Uses `youtipie` package (custom YouTube client library).
- Supports YouTube search, channel subscriptions, playlist parsing, video streams, audio-only streams, comments, and comment replies (`yt_miniplayer_comments_subpage.dart`).
- Supports background video conversion/muxing using `ffmpeg_kit_flutter`.

### 4.5 Native Tag Editor (`lib/controller/tagger_controller.dart`, `flutter_taglib`)
- Binds to `TagLib` C++ library via FFI (`flutter_taglib`).
- Allows editing embedded cover art, artist names, track titles, album details, and custom metadata directly on FLAC, MP3, M4A, and OGG files on disk.

---

## 5. UI Architecture & Design Details

- **Dynamic Visuals**: Uses `animated_background` combined with `palette_generator` to create smooth, shifting particle and mesh gradients behind the current playing track art.
- **3D Card Tilt**: Uses `flutter_tilt` to give album cover artwork a physical 3D tilt reaction based on touch/mouse position.
- **Material You Engine**: Integrates `dynamic_color` for system-wide Material 3 accent matching on Android 12+.
- **Virtualization**: Uses `super_sliver_list` for high-performance rendering of deep music libraries with smooth 60fps scrolling.

---

## 6. Key Takeaways for Rustic Player

1. **Multi-Source Architecture**: Demonstrates how to cleanly unify local files, WebDAV, Subsonic, SMB, and YouTube streams under a single `Track` model.
2. **TagLib FFI Integration**: Using native TagLib for fast metadata reading/writing is far superior to pure-Dart tag parsers.
3. **Audio Waveforms**: Pre-calculating waveform peak arrays provides a visually impressive seekbar.
