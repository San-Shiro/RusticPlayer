# Phase 02 — Library + UI Atoms/Molecules

**Tracks:** A (E2) · B (U1)

## Objective
Unified multi-source SQLite library with scanning, search, artwork, waveforms. UI kit gains all atoms and molecules with complete states.

## Track A — Engine (library)
Libraries: `rusqlite`(bundled, FTS5) · `jwalk` · `lofty` · `blake3` · `image`/`webp`. From scratch: schema, scanner, artwork pipeline, waveform.

- [ ] Schema + migrations: `tracks(source, isrc, content_hash…)` · `albums` · `artists` · `playlists` · `playlist_tracks` · `source_matches(track_key, source_slug, video_id, score, fetched_at)` · `stream_cache_meta` · FTS5(title/artist/album)
- [ ] Scanner: jwalk parallel walk → lofty tags → mtime/size incremental rescan → blake3 of decoded audio (Jam Mode B seam)
- [ ] Tag fallbacks: filename parse, multi-disc, compilations (album artist), various-artists
- [ ] Artwork: embedded + remote fetch → 64/256/640px WebP thumbs, content-hash dedupe, bounded LRU (20 full / 200 thumbs)
- [ ] `library/waveform.rs` — off-thread peak extraction → cached normalized `[u8; 256]` per track
- [ ] Search: FTS5, case/accent-insensitive, <100ms on 10k tracks

**Tests:** 50-file fixture (unicode/CJK/emoji, no tags, multi-disc, compilation, dup titles) · rescan-unchanged = zero file reads (instrumented) · one-tag-change = one row update · delete removes · search unicode/partial · 10k scan <60s bench · thumbnail cache-hit + dedupe.

## Track B — UI (atoms + molecules)
- [ ] Atoms: Button, IconButton, SeekBar (flat + waveform variant), VolumeSlider, Marquee, Artwork (shimmer + crossfade), SourceChip, Skeleton, Badge, Switch, TextField (search w/ debounce hook), Tooltip, ContextMenu, Avatar
- [ ] Molecules: TrackRow (hover actions), AlbumCard (hover play overlay), ArtistCard, PlaylistCard, TransportControls (repeat tri-state), QueueItem (drag handle + attribution avatar), LyricsLine (word segments), SectionHeader
- [ ] Every state in gallery: default/hover/pressed/disabled/loading/empty

## Done when
`cli scan ~/Music` + `cli search` on 10k files; rescan near-instant; gallery shows every atom/molecule in every state.
