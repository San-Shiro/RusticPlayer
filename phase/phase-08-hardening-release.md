# Phase 08 — Hardening & Release

**Tracks:** A (E9) · B (release) · **Exit: S4 — release candidate**

## Objective
Turn the working product into a shippable one: packaged installs on three OSes, 8-hour soak clean, first-run experience smooth.

## Track A — Engine (hardening)
- [ ] 8h continuous mixed-source playback soak: no crash, RSS flat, no handle leaks
- [ ] Error taxonomy pass: `thiserror` in every crate; every `Event::Error` is actionable; zero user-facing panics
- [ ] Opt-in crash reporting
- [ ] Headless receiver packaging: `cli --receiver` + systemd unit / Windows service notes
- [ ] Cold-start instrumentation on packaged builds

## Track B — UI (release)
- [ ] First-run experience: Onboarding → folder pick + provider login → import progress → Home populated
- [ ] Auto-update check (opt-in)
- [ ] Packaging assets: `.desktop` file, icons (all sizes), installer branding
- [ ] Final a11y pass: keyboard-only navigation of every page, focus rings, ⌘K palette coverage
- [ ] Resize sweep 600px → 4K, fractional scaling 100/125/150/200%, Wayland + X11

## Packaging matrix
| OS | Format | Notes |
|---|---|---|
| Linux | AppImage + **Flatpak** | Flatpak is the distribution priority; verify no missing runtime deps on clean Ubuntu/Fedora/Arch VMs |
| Windows | MSI or NSIS | Code signing |
| macOS | `.app` bundle | Notarization |

## S4 release gates (all must pass)
- [ ] 8h soak: RSS flat, no crash
- [ ] Idle ~0% CPU (window open, nothing playing)
- [ ] <150MB RSS after browsing 50 albums
- [ ] 10k-track scroll p99 <8ms; keystroke→results <100ms
- [ ] Cold start to first pixel <150ms
- [ ] Install → play → uninstall leaves no orphans, on all three OSes
- [ ] Jam: 3 devices Mode A party test passes

## Post-v1 (deferred, seams already cut)
Mobile thin client (`ui-plan.md` §6) · WASM plugins (`plugin-api`) · JioSaavn/Subsonic/WebDAV/SMB providers · AI playlists · Canvas videos.
