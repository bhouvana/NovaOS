# tests/

Status: **Not yet implemented** — Phase 1 (architecture only). Landed alongside the
CI pipeline in Phase 2
([../docs/12-ROADMAP-AND-MILESTONES.md](../docs/12-ROADMAP-AND-MILESTONES.md) §3).

Cross-crate integration, end-to-end, and system tests that don't belong beside a single
crate: full-image headless-QEMU boot tests, SDK contract tests, the browser-demo smoke
test. Unit tests live beside the crates they test, not here — see
[../docs/10-TESTING-AND-BUILD.md](../docs/10-TESTING-AND-BUILD.md) §3 for the full
testing pyramid and what belongs at each level.
