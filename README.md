# NovaOS

**A lightweight desktop platform built on Linux** — not another Linux distro, and not
positioned as one. The engineering effort here goes into the desktop, the developer
platform, and the user experience: its own compositor, its own UI toolkit, its own app
suite, its own SDK, its own package system. Linux supplies the kernel and drivers;
everything the user or a developer touches is NovaOS. Boots on real hardware, in a VM,
and directly in your browser.

**Status**: Phase 2 — Vertical Slice, implemented and tested; Phase 2.5 (System
Validation) not yet performed. Phases 1 and 1.5 (architecture, interaction flows, wire
protocols, per-service RFCs, byte-level formats) are done. Phase 2's real, working code
so far: Nova Bus (`services/nova-bus`, `services/nova-bus-broker`), the Nova SDK
(`sdk/nova-app`, `sdk/nova-ui`), and a Hello World app (`apps/hello`) proven end-to-end
as real, separate OS processes by `tests/vertical-slice`. The compositor, real boot, and
browser demo remain environment-blocked (no Linux graphics toolchain in this
development setup — see
[docs/12-ROADMAP-AND-MILESTONES.md](docs/12-ROADMAP-AND-MILESTONES.md) §4's Environment
note). See [docs/IMPLEMENTATION-NOTES/](docs/IMPLEMENTATION-NOTES/) for every place
running code has diverged from the specs, including one security bug found and fixed
during implementation.

## Start Here

- [docs/00-VISION.md](docs/00-VISION.md) — what NovaOS is and why it exists
- [docs/ENGINEERING-PRINCIPLES.md](docs/ENGINEERING-PRINCIPLES.md) — the short list
  every contribution is judged against
- [docs/01-SYSTEM-ARCHITECTURE.md](docs/01-SYSTEM-ARCHITECTURE.md) — the layered system
  architecture and boot sequence
- [docs/02-REPOSITORY-STRUCTURE.md](docs/02-REPOSITORY-STRUCTURE.md) — how this repo is
  organized and why
- [docs/decisions/](docs/decisions/) — Architecture Decision Records for every
  significant technology choice
- [docs/rfcs/README.md](docs/rfcs/README.md) — the RFC process, and per-service
  contracts for every core Nova service
- [docs/IMPLEMENTATION-NOTES/README.md](docs/IMPLEMENTATION-NOTES/README.md) — where
  running code has diverged from the specs, why, and what closes the gap
- [docs/specs/00-INDEX.md](docs/specs/00-INDEX.md) — the Phase 1.5 engineering
  specification: interaction flows, wire protocols, memory/boot budgets, and full
  subsystem specs
- [docs/12-ROADMAP-AND-MILESTONES.md](docs/12-ROADMAP-AND-MILESTONES.md) — the phased
  plan from here to a v1.0 release

## Full Documentation Index

### Phase 1 — Architecture

| Doc | Covers |
|---|---|
| [00-VISION.md](docs/00-VISION.md) | Vision, product goals, success criteria |
| [01-SYSTEM-ARCHITECTURE.md](docs/01-SYSTEM-ARCHITECTURE.md) | Layers, boot sequence, process topology, tech stack |
| [02-REPOSITORY-STRUCTURE.md](docs/02-REPOSITORY-STRUCTURE.md) | Folder layout, ownership, dependency rules |
| [03-DESKTOP-ARCHITECTURE.md](docs/03-DESKTOP-ARCHITECTURE.md) | Compositor, launcher, taskbar, notifications, session management |
| [04-APPLICATION-FRAMEWORK-AND-SDK.md](docs/04-APPLICATION-FRAMEWORK-AND-SDK.md) | App lifecycle, SDK modules, manifests, plugins |
| [05-PACKAGE-AND-UPDATE-SYSTEM.md](docs/05-PACKAGE-AND-UPDATE-SYSTEM.md) | `.novapkg` format, Package Center, A/B OS updates |
| [06-DESIGN-SYSTEM.md](docs/06-DESIGN-SYSTEM.md) | Tokens, motion, components, theming, accessibility |
| [07-BROWSER-DEPLOYMENT.md](docs/07-BROWSER-DEPLOYMENT.md) | novaos.dev, v86 in-browser boot strategy |
| [08-SECURITY-MODEL.md](docs/08-SECURITY-MODEL.md) | Permissions, sandboxing, secrets, accounts |
| [09-PERFORMANCE-STRATEGY.md](docs/09-PERFORMANCE-STRATEGY.md) | RAM/boot/frame-time budgets and enforcement |
| [10-TESTING-AND-BUILD.md](docs/10-TESTING-AND-BUILD.md) | Build system, CI pipeline, testing pyramid |
| [11-CODING-STANDARDS.md](docs/11-CODING-STANDARDS.md) | Conventions, versioning, branching, release process |
| [12-ROADMAP-AND-MILESTONES.md](docs/12-ROADMAP-AND-MILESTONES.md) | Phase 1–6 plan and exit criteria |
| [13-RISK-ASSESSMENT.md](docs/13-RISK-ASSESSMENT.md) | Known risks and mitigations |
| [14-FUTURE-VISION.md](docs/14-FUTURE-VISION.md) | Deferred ideas and extensibility surfaces |

### Phase 1.5 — Engineering Specification

| Doc | Covers |
|---|---|
| [specs/00-INDEX.md](docs/specs/00-INDEX.md) | Index + Phase 1 ↔ 1.5 consistency check |
| [specs/01-INTERACTION-FLOWS.md](docs/specs/01-INTERACTION-FLOWS.md) | Sequence diagrams for every cross-subsystem flow |
| [specs/02-MEMORY-BUDGET.md](docs/specs/02-MEMORY-BUDGET.md) | Fully itemized RAM ledger |
| [specs/03-BOOT-TIMELINE.md](docs/specs/03-BOOT-TIMELINE.md) | Millisecond-level VM + hardware boot timelines |
| [specs/04-WINDOW-MANAGER-SPEC.md](docs/specs/04-WINDOW-MANAGER-SPEC.md) | Window lifecycle, focus, z-order, damage tracking, frame scheduling |
| [specs/05-NOVA-UI-TOOLKIT-SPEC.md](docs/specs/05-NOVA-UI-TOOLKIT-SPEC.md) | Widget tree, layout algorithm, rendering pipeline, theming |
| [specs/06-NOVA-SDK-SPEC.md](docs/specs/06-NOVA-SDK-SPEC.md) | Full app model, manifest schema, every SDK API |
| [specs/07-PACKAGE-FORMAT-SPEC.md](docs/specs/07-PACKAGE-FORMAT-SPEC.md) | Byte-level `.novapkg` layout |
| [specs/08-BROWSER-ARCHITECTURE-SPEC.md](docs/specs/08-BROWSER-ARCHITECTURE-SPEC.md) | novaos.dev stack, input/clipboard/persistence/network bridging |
| [specs/09-APPLICATION-SPECS.md](docs/specs/09-APPLICATION-SPECS.md) | Per-app design docs |
| [specs/10-DESIGN-BIBLE.md](docs/specs/10-DESIGN-BIBLE.md) | Concrete, final design token values |
| [specs/11-BUILD-PIPELINE-SPEC.md](docs/specs/11-BUILD-PIPELINE-SPEC.md) | Source-to-artifact build pipeline |
| [specs/12-BROWSER-DEMO-EXPERIENCE.md](docs/specs/12-BROWSER-DEMO-EXPERIENCE.md) | The novaos.dev demo UX |
| [specs/13-WEBSITE-INFORMATION-ARCHITECTURE.md](docs/specs/13-WEBSITE-INFORMATION-ARCHITECTURE.md) | Full site map |
| [specs/14-ECOSYSTEM-VISION.md](docs/specs/14-ECOSYSTEM-VISION.md) | Store/Cloud/Sync/Community chain, reconciled against v1 non-goals |
| [specs/15-NOVA-BUS-PROTOCOL-SPEC.md](docs/specs/15-NOVA-BUS-PROTOCOL-SPEC.md) | Complete Nova Bus wire protocol: envelope, framing, timeouts, error codes, auth |
| [specs/16-STATE-MACHINES.md](docs/specs/16-STATE-MACHINES.md) | Every subsystem's state machine in one place |
| [specs/17-SDK-API-REFERENCE-POLICY.md](docs/specs/17-SDK-API-REFERENCE-POLICY.md) | SDK stability tiers, deprecation policy, error conventions |
| [specs/18-PLUGIN-ARCHITECTURE-SPEC.md](docs/specs/18-PLUGIN-ARCHITECTURE-SPEC.md) | Plugin discovery, nested sandboxing, version negotiation |
| [specs/19-FILESYSTEM-LAYOUT-SPEC.md](docs/specs/19-FILESYSTEM-LAYOUT-SPEC.md) | The complete on-disk directory tree |
| [specs/20-CONFIGURATION-STRATEGY-SPEC.md](docs/specs/20-CONFIGURATION-STRATEGY-SPEC.md) | TOML decision, config scopes, live reload, schema evolution |
| [specs/21-OBSERVABILITY-SPEC.md](docs/specs/21-OBSERVABILITY-SPEC.md) | Logs, metrics, tracing, debug overlays, health checks |
| [specs/22-RELEASE-PROCESS-SPEC.md](docs/specs/22-RELEASE-PROCESS-SPEC.md) | Release channels, signing hierarchy, rollback, changelogs |

### RFCs — Per-Service Contracts

| RFC | Service |
|---|---|
| [rfcs/README.md](docs/rfcs/README.md) | The RFC process itself, and ADR-vs-RFC guidance |
| [RFC-0001](docs/rfcs/RFC-0001-nova-shell.md) | Nova Shell |
| [RFC-0002](docs/rfcs/RFC-0002-nova-bus.md) | Nova Bus |
| [RFC-0003](docs/rfcs/RFC-0003-nova-wm.md) | Nova WM (Compositor) |
| [RFC-0004](docs/rfcs/RFC-0004-package-service.md) | Package Service |
| [RFC-0005](docs/rfcs/RFC-0005-notification-service.md) | Notification Service |
| [RFC-0006](docs/rfcs/RFC-0006-theme-service.md) | Theme Service |
| [RFC-0007](docs/rfcs/RFC-0007-settings-service.md) | Settings Service |
| [RFC-0008](docs/rfcs/RFC-0008-session-manager.md) | Session Manager |
| [RFC-0009](docs/rfcs/RFC-0009-update-service.md) | Update Service |

## Repository Layout

```
system/    Base system, boot, init, image build, updates
services/  Resident Nova Services (IPC bus, session manager, package manager, ...)
desktop/   Compositor + desktop shell (launcher, taskbar, notifications, settings)
sdk/       Nova SDK — the only thing apps are allowed to depend on
apps/      First-party Nova applications and Nova Arcade games
web/       novaos.dev — browser demo (v86) and documentation site
tools/     Build/CI/dev tooling
tests/     Cross-crate integration and system tests
docs/      Everything above
```

See [docs/02-REPOSITORY-STRUCTURE.md](docs/02-REPOSITORY-STRUCTURE.md) for the full
layout and the dependency rules CI enforces between these folders. Also present now:
`tests/vertical-slice` (the multi-process proof described above) and
`tools/nova-bus-bench` (real latency/throughput measurement,
[docs/IMPLEMENTATION-NOTES/0005](docs/IMPLEMENTATION-NOTES/0005-nova-bus-measured-performance.md)).

## Building & Testing

Requires Rust (stable) and `protoc` (the Protobuf compiler — `services/nova-bus`'s
build script needs it; see
[docs/IMPLEMENTATION-NOTES/0001](docs/IMPLEMENTATION-NOTES/0001-nova-bus-dev-transport-tcp.md)
for the platform note on this vertical slice's dev transport).

```sh
cargo build --workspace
cargo test --workspace
cargo run --release -p nova-bus-bench   # real latency/throughput numbers
```

If `protoc` isn't on `PATH`, set the `PROTOC` environment variable to its full path.
No Linux graphics toolchain (wlroots/QEMU) is required to build or test anything above
— those are only needed for `nova-compositor` and full-image work, which don't exist
yet ([docs/12-ROADMAP-AND-MILESTONES.md](docs/12-ROADMAP-AND-MILESTONES.md) §4).

## Principles

Simplicity, maintainability, consistency, performance, developer experience, low memory,
beautiful UX, modularity — in that priority order when two conflict. See
[docs/00-VISION.md](docs/00-VISION.md) §6.

## Contributing

Not yet open for broad external contribution — the vertical slice
([docs/12-ROADMAP-AND-MILESTONES.md](docs/12-ROADMAP-AND-MILESTONES.md) §4) is real and
buildable, but Phase 2.5 (System Validation) hasn't happened yet, so the SDK/protocol
surface may still change based on what that review finds. See
[CONTRIBUTING.md](CONTRIBUTING.md) for setup and process. Every non-trivial
architectural change requires an RFC and/or ADR before implementation
([docs/rfcs/README.md](docs/rfcs/README.md),
[docs/11-CODING-STANDARDS.md](docs/11-CODING-STANDARDS.md) §8) — this applies to the
core team's own ongoing work, not only external contributors.

## License

TBD — recorded as an open decision; a permissive/copyleft choice will be captured as an
ADR before Phase 2 accepts external contributions.
