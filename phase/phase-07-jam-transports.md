# Phase 07 — Jam P2P & Cast-Receiver + UI Polish

**Tracks:** A (E8) · B (U7)

## Objective
Jam works beyond LAN: WebRTC P2P with code pairing, and a cast-receiver mode that doubles as the WebRTC-failure fallback — no mandatory central server. UI reaches release-level polish.

## Track A — Engine (transports)
Libraries: `webrtc` (pure-Rust Pion port) · `snow` · `tokio-tungstenite`. From scratch: rendezvous protocol, receiver mode.

- [ ] `jam/transports/p2p.rs` — WebRTC data channels + STUN; code-pairing signaling via minimal rendezvous (one POST/GET pair, self-hostable ~100 lines); E2E key derived from code **before** signaling sees anything
- [ ] `jam/transports/cast_receiver.rs` — receiver mode: headless `cli --receiver` (e.g. Raspberry Pi on speakers) advertises by code; senders drive it via the same Command/Event wire protocol; **automatic fallback when WebRTC negotiation fails** (relayed WS via rendezvous) — self-hosted YouTube-Cast model
- [ ] `TransportCaps.latency_class` gates Mode B: only `p2p`/`lan` offer sync playback
- [ ] Mode B (if caps allow): SNTP-style offset (min-RTT window of ~20, resync 5s, smoothed) · drift correction by ±0.05–0.1% `rubato` resample, never drop samples · guest buffer 300–1000ms · audio delivery: same-file by content hash, else host Opus stream
- [ ] Transport swap is invisible to session layer + UI (pluggable-for-testing requirement)

**Tests:** WebRTC-fail → cast-receiver fallback transparent to UI · `tc netem` 50/100/200ms jitter stays smooth · mid-track join syncs <2s · 30-min sustained sync bounded drift (Mode B) · two devices same room <5ms offset measured by mic (Mode B).

## Track B — UI (polish)
- [ ] Context menus everywhere (track/album/playlist/queue)
- [ ] Drag-drop: playlist reorder + queue, autoscroll, edge cases (drop on self/outside/Esc)
- [ ] Marquee on all overflowing titles; loading skeletons final
- [ ] Light/dark + system preference; theme switch mid-playback without flicker
- [ ] Connection/sync-health indicators (drift ms behind dev flag)
- [ ] Every Jam failure state has UI: host left · network dropped · kicked · session full · version mismatch
- [ ] Re-run ALL perf gates: idle ~0% · <150MB after 50 albums · 10k scroll p99 <8ms · startup <150ms

## Done when
Jam Mode A works on LAN + P2P + cast-receiver with transparent fallback; (Mode B where caps allow); the app looks screenshot-worthy and idle CPU is still ~0%.
