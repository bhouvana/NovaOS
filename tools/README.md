# tools/

Status: **Not yet implemented** — Phase 1 (architecture only). CI scripts and the image
builder land in Phase 2
([../docs/12-ROADMAP-AND-MILESTONES.md](../docs/12-ROADMAP-AND-MILESTONES.md) §3).

Developer and CI tooling: the image-builder CLI (assembles `system/image/` recipes into
bootable artifacts), the dev VM launcher, lint configs (dependency-direction lint,
ADR/doc lint), and the `nova-cli new` app scaffolding tool (Phase 5). See
[../docs/10-TESTING-AND-BUILD.md](../docs/10-TESTING-AND-BUILD.md) for the CI pipeline
these scripts implement.
