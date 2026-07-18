# NovaOS — Phase 1.5 Engineering Specification

Status: Draft v0.1 · Owner: Chief Architect · Last updated: 2026-07-18

## Purpose

Phase 1 ([../00-VISION.md](../00-VISION.md) through [../14-FUTURE-VISION.md](../14-FUTURE-VISION.md))
established *what* NovaOS is and *why* each major technology was chosen. It deliberately
stopped at the layer-diagram level of detail.

That is not enough to start Phase 2 without risking rework: two engineers independently
implementing "the compositor talks to the session manager" or "the package format has a
manifest and a signature" will make different, incompatible micro-decisions — exact
message order, exact byte layout, exact field names — that only surface as integration
bugs later. This tree closes that gap. Nothing here overturns a Phase 1 ADR; everything
here is Phase 1 made concrete enough to type into an editor without guessing.

## Document Set

| # | Doc | Answers |
|---|---|---|
| 01 | [Interaction Flows](01-INTERACTION-FLOWS.md) | For each cross-subsystem action, exactly which components talk to which, in what order, with what messages |
| 02 | [Memory Budget](02-MEMORY-BUDGET.md) | Where every megabyte of the 64–100 MB idle budget actually goes |
| 03 | [Boot Timeline](03-BOOT-TIMELINE.md) | Millisecond-by-millisecond accounting of the boot budget on VM and reference hardware |
| 04 | [Window Manager Spec](04-WINDOW-MANAGER-SPEC.md) | Window lifecycle/state machine, focus, z-order, damage tracking, frame scheduling |
| 05 | [Nova UI Toolkit Spec](05-NOVA-UI-TOOLKIT-SPEC.md) | Widget tree, layout algorithm, rendering pipeline, animation system, theming |
| 06 | [Nova SDK Spec](06-NOVA-SDK-SPEC.md) | Full app model, manifest schema, every SDK API surface with signatures |
| 07 | [Package Format Spec](07-PACKAGE-FORMAT-SPEC.md) | Byte-level `.novapkg` layout: header, manifest, assets, signature |
| 08 | [Browser Architecture Spec](08-BROWSER-ARCHITECTURE-SPEC.md) | novaos.dev stack end-to-end: site → canvas → WASM → v86 → guest, and every I/O bridge |
| 09 | [Application Specs](09-APPLICATION-SPECS.md) | Per-app design: purpose, window model, views, data model, SDK usage, permissions |
| 10 | [Design Bible](10-DESIGN-BIBLE.md) | Concrete, final values for every design token — not names, numbers |
| 11 | [Build Pipeline Spec](11-BUILD-PIPELINE-SPEC.md) | Every stage from `cargo build` to a published ISO/browser-demo artifact |
| 12 | [Browser Demo Experience](12-BROWSER-DEMO-EXPERIENCE.md) | The actual UX of landing on novaos.dev and clicking around |
| 13 | [Website Information Architecture](13-WEBSITE-INFORMATION-ARCHITECTURE.md) | Full site map: every page, its purpose, its content source |
| 14 | [Ecosystem Vision](14-ECOSYSTEM-VISION.md) | How Store/Cloud/Sync/Community extend the platform without violating v1 non-goals |
| 15 | [Nova Bus Protocol Spec](15-NOVA-BUS-PROTOCOL-SPEC.md) | Complete wire protocol: envelope, framing, request/response, pub/sub, versioning, auth |
| 16 | [State Machines](16-STATE-MACHINES.md) | Every subsystem's explicit state machine in one place |
| 17 | [SDK API Reference Policy](17-SDK-API-REFERENCE-POLICY.md) | Stability tiers, deprecation policy, semver rules, error conventions |
| 18 | [Plugin Architecture Spec](18-PLUGIN-ARCHITECTURE-SPEC.md) | Discovery, loading, nested sandboxing, permission intersection, version negotiation |
| 19 | [Filesystem Layout Spec](19-FILESYSTEM-LAYOUT-SPEC.md) | The complete `/nova/` directory tree: ownership, persistence, access rules |
| 20 | [Configuration Strategy Spec](20-CONFIGURATION-STRATEGY-SPEC.md) | TOML decision, config scopes, live reload, validation, schema evolution |
| 21 | [Observability Spec](21-OBSERVABILITY-SPEC.md) | Structured logs, metrics, tracing, debug overlays, health checks |
| 22 | [Release Process Spec](22-RELEASE-PROCESS-SPEC.md) | Nightly/beta/stable channels, signing hierarchy, rollback, changelog generation |

Also see [../rfcs/](../rfcs/) — per-service RFCs (Nova Shell, Nova Bus, Nova WM, Package
Service, Notification Service, Theme Service, Settings Service, Session Manager, Update
Service) and the RFC process itself, and
[../ENGINEERING-PRINCIPLES.md](../ENGINEERING-PRINCIPLES.md) — the capstone principles
every RFC/ADR is checked against.

## Ground Rules

- **No new ADRs are silently implied here.** If a spec in this tree requires a decision
  Phase 1 didn't make (e.g., "what frontend framework does novaos.dev use"), it's called
  out explicitly and treated as a decision, with rationale — see
  [08-BROWSER-ARCHITECTURE-SPEC.md](08-BROWSER-ARCHITECTURE-SPEC.md) §1 for the one case
  that came up (site framework choice).
- **Every number is derived, not asserted.** The memory and boot budgets in
  [../09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md) stated targets; 02 and
  03 here show the itemized math that the target is actually reachable, and become the
  literal checklist CI's regression gates are written against.
- **This tree is illustrative-precision, not final-implementation-precision.** Struct
  field names, message sequences, and numeric budgets here are specific enough to code
  against and are expected to hold, but Phase 2 implementation may reveal a specific
  field needs to change — that's a normal PR updating this doc alongside the code
  ([../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md) §7), not a Phase 1.5 failure.
- **Status per doc** is tracked in that doc's own header. All 14 are drafted in this
  pass; the roadmap's Phase 1.5 exit criteria
  ([../12-ROADMAP-AND-MILESTONES.md](../12-ROADMAP-AND-MILESTONES.md) §3) is met once
  they're internally consistent — cross-checked in §"Consistency Check" below.

## Consistency Check (Phase 1 ↔ Phase 1.5)

| Phase 1 claim | Phase 1.5 doc that proves/derives it |
|---|---|
| Idle RAM 64–100 MB ([00-VISION.md](../00-VISION.md) §5) | [02-MEMORY-BUDGET.md](02-MEMORY-BUDGET.md) itemizes to the same range |
| Boot ≤8s hardware / ≤5s VM ([09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md) §2) | [03-BOOT-TIMELINE.md](03-BOOT-TIMELINE.md) sums to within both budgets with headroom |
| Nova Bus request/response + pub/sub ([ADR-0006](../decisions/ADR-0006-ipc-mechanism.md)) | [01-INTERACTION-FLOWS.md](01-INTERACTION-FLOWS.md) shows both message modes in real flows |
| `.novapkg` = SquashFS + manifest, signed ([ADR-0007](../decisions/ADR-0007-package-format.md)) | [07-PACKAGE-FORMAT-SPEC.md](07-PACKAGE-FORMAT-SPEC.md) gives the exact byte layout |
| Compositor-drawn permission prompts ([08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §3) | [01-INTERACTION-FLOWS.md](01-INTERACTION-FLOWS.md) §Permission Prompt Flow traces it message-by-message |
| Browser demo, no backend VM fleet ([ADR-0009](../decisions/ADR-0009-browser-boot-emulator.md)) | [08-BROWSER-ARCHITECTURE-SPEC.md](08-BROWSER-ARCHITECTURE-SPEC.md) confirms every I/O path is client-side-only |
| Every resident service has a documented contract, not just a box in a diagram | [../rfcs/](../rfcs/) — 9 RFCs, one per core service, added in the second Phase 1.5 review round |
| No cloud-account requirement anywhere in v1 ([../00-VISION.md](../00-VISION.md) §7) | [14-ECOSYSTEM-VISION.md](14-ECOSYSTEM-VISION.md) §3's three binding constraints on any future Cloud/Sync work |
| Browser demo is the real OS, not a second UI implementation | [08-BROWSER-ARCHITECTURE-SPEC.md](08-BROWSER-ARCHITECTURE-SPEC.md) §1b's structural (not just conventional) React/NovaOS rendering boundary |
