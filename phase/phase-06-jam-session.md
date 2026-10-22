# Phase 06 — Jam Session Layer + UI Panels & Animation Pass

**Tracks:** A (E7) · B (U6) · **Exit: S3 — LAN Jam works end-to-end**

## Objective
Host-authoritative Jam sessions over loopback + LAN transports, with the Jam UI panel live. Full animation token pass across the app.

## Track A — Engine (jam session)
Libraries: `mdns-sd` · `tokio-tungstenite` · `snow` (Noise). From scratch: session layer, loopback transport.

- [ ] `jam/session.rs` — host-authoritative (no CRDTs); roles Host/Guest/Listener; guest Commands are requests, host validates + applies + broadcasts Events
- [ ] 6-char join codes + ephemeral keypair; handshake: code → ECDH → `snow` session key (E2E regardless of transport); join rate-limit
- [ ] Attribution: queue items carry `origin: PeerId`; per-guest pending-add limit (3); host-wins conflict rule
- [ ] Full-snapshot resync on reconnect (no diffs); `PROTOCOL_VERSION` gate with readable rejection
- [ ] `jam/transports/loopback.rs` — in-process channel pair; ALL session tests run here
- [ ] `jam/transports/lan.rs` — mDNS `_rusticjam._tcp.local` advertise/discover + WebSocket control channel

**Tests (all on loopback first, then two machines on LAN):** wrong-code rejected + brute-force rate-limited · guest add visible on host <200ms · host disconnect → clean "session ended" · guest drop mid-action → host consistent · 8-guest flood → deterministic convergence.

## Track B — UI (panels + animation)
- [ ] JamPanel: host view (code + QR + participants with roles, kick, lock queue) · guest view (nearby sessions list + code entry) · status pill (connected/reconnecting/degraded)
- [ ] Queue attribution avatars on live data
- [ ] Motion token pass per `ui-plan.md` §7: page transitions (fade+8px rise, 300ms emphasized) · sheet springs (260/26) · press scale 0.97 · marquee 40px/s · artwork crossfade 250ms · palette crossfade 300ms
- [ ] `prefers-reduced-motion` path; blur cache validation (static blur delta ≈ 0)

## Done when
Three devices on one LAN: host plays, guests add/vote, queue converges everywhere; every animation honors tokens; tracy shows transition p99 <8ms → **S3.**
