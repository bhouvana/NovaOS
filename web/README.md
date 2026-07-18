# web/

Status: **Not yet implemented** — Phase 1 (architecture only). Planned for Phase 6
([../docs/12-ROADMAP-AND-MILESTONES.md](../docs/12-ROADMAP-AND-MILESTONES.md) §7).

novaos.dev: the static, CDN-hostable site that boots the real NovaOS ISO in-browser via
the v86 WASM x86 emulator, hosts the downloadable installable ISO, and serves the
generated documentation site. No backend/server process — full design and rationale in
[../docs/07-BROWSER-DEPLOYMENT.md](../docs/07-BROWSER-DEPLOYMENT.md) and
[ADR-0009](../docs/decisions/ADR-0009-browser-boot-emulator.md).

Depends only on build artifacts (the built ISO / browser-demo image) produced by
`system/image/` and the generated SDK docs — never depends on source crates directly
([../docs/02-REPOSITORY-STRUCTURE.md](../docs/02-REPOSITORY-STRUCTURE.md) §3 rule 6).
