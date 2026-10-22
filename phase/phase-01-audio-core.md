# Phase 01 — Audio Core + UI Kit Scaffold

**Tracks:** A (E1, hardest engine phase) · B (U0)

## Objective
Engine plays local files and ranged-HTTP streams gaplessly with crossfade, headless via CLI. UI kit crate boots with theming and a component gallery.

## Track A — Engine (audio)
Libraries: `symphonia`(mkv/mp4/ogg/flac/aac) · `cpal` · `rtrb` · `rubato` · `reqwest`(rustls). From scratch: http_source, stream_cache, crossfade mixer.

- [ ] `audio/source.rs` — `MediaSource` abstraction: `FileSource` + `HttpSource` (HTTP range, 64KB blocks, retry+backoff)
- [ ] `audio/stream_cache.rs` — write-through chunk cache keyed by (video_id|url, itag); meta in SQLite; replay = disk only
- [ ] `audio/decoder.rs` — probe → decode f32 interleaved → resample to device rate; corrupt files return `Err`, never panic
- [ ] `audio/output.rs` — cpal stream, 500ms rtrb ring, AtomicF32 volume + 5ms smoothing ramp, underrun counter → silence (never glitch-loop)
- [ ] `audio/transport.rs` — `Stopped|Playing|Paused|Buffering`; seek = flush ring + reposition; position from **samples consumed by callback**
- [ ] `audio/crossfade.rs` — pipeline B pre-decodes next track; equal-power cos/sin ramp; configurable 0–12s
- [ ] ReplayGain: parse tags → gain staging in mixer
- [ ] Callback hard rule: no alloc / lock / log — enforce with a panicking test allocator

**Tests:** codec corpus + broken corpus + proptest garbage · 100-seek storm · two-tone gapless click test · wiremock ranged-Opus playback, then replay with server killed (cache hit) · device disconnect recovery.

## Track B — UI (kit scaffold)
- [ ] `crates/ui-kit`: depends only on gpui + gpui-component; no engine import, no I/O
- [ ] `tokens.rs`: Palette (dark/light) + dynamic-palette slot, TypeScale, 4pt Spacing, Radii, Elevation, Motion
- [ ] `examples/gallery.rs` — renders any component in isolation; theme switch live

## Done when
`cli play <path>` and `cli play <http-url>` both gapless with crossfade; 10-min glitch-free; gallery binary runs and hot-swaps themes.
