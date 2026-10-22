# Rustic Player — Phased Execution

One folder entry per phase. Each phase file merges **Track A (core engine)** and **Track B (UI)** work that runs in parallel, with gates that must pass before the next phase starts.

**Source docs:** `plans/rustic-player-master-plan.md` (strategy) · `plans/core-engine-plan.md` (Track A detail) · `plans/ui-plan.md` (Track B detail) · `task.md` (live checklist).

| Phase | Name | Track A (engine) | Track B (UI) | Exits on |
|---|---|---|---|---|
| [00](phase-00-contracts.md) | Contracts & Foundation | E0 | prep | **S0** contract freeze |
| [01](phase-01-audio-core.md) | Audio Core + Kit Scaffold | E1 | U0 | glitch-free local+HTTP playback; gallery runs |
| [02](phase-02-library.md) | Library + Atoms/Molecules | E2 | U1 | 10k scan/search; all component states |
| [03](phase-03-sources.md) | Sources + Organisms | E3 | U2 | YTM+Spotify login→import→play; virtualized lists |
| [04](phase-04-integration.md) | Integration + Shell/Pages | E4 | U3–U4 | **S1** MockEngine · **S2** daily-driver |
| [05](phase-05-services-platform.md) | Services & Platform + Live UI | E5–E6 | U5 | lyrics/SponsorBlock/scrobble live; MPRIS/SMTC |
| [06](phase-06-jam-session.md) | Jam Session + Panels/Animation | E7 | U6 | **S3** LAN jam works end-to-end |
| [07](phase-07-jam-transports.md) | Jam Transports + Polish | E8 | U7 | P2P + cast-receiver fallback; perf gates |
| [08](phase-08-hardening-release.md) | Hardening & Release | E9 | release | **S4** RC gates: 8h soak, <150MB, ~0% idle |

**Parallelism rule:** the `Command`/`Event` contract (frozen in Phase 00) is the only coupling. UI builds against MockEngine until Phase 04; engine ships phases headless via CLI until then.

**Every phase, both tracks:** idle-CPU check · memory-flat soak · tests green on Linux + Windows + macOS CI.
