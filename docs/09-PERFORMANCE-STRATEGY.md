# NovaOS — Performance Strategy

Status: Draft v0.1 · Owner: Chief Architect · Last updated: 2026-07-18

## 1. Philosophy

Measure first, optimize the measured bottleneck, never guess. Every budget below is a
number that must be produced by an automated measurement in CI or on reference hardware,
not estimated. A budget with no measurement behind it is a wish, not a target.

## 2. Budgets

| Metric | Target | Measured on | Enforcement |
|---|---|---|---|
| Idle RAM (desktop shown, no apps open) | 64–100 MB | Reference VM (2 vCPU, defined RAM cap) + reference low-end hardware | CI regression check on every release candidate |
| Cold boot (firmware handoff → desktop ready) | ≤ 8s on reference hardware, ≤ 5s in reference VM | Boot-milestone timestamps (§3) | Release-blocking |
| App launch (Package Center click → window visible) | ≤ 500ms for first-party apps | `nova-sessiond` launch-to-first-frame timer | Tracked per app, regression-alerted |
| Compositor frame time | ≤ 16.6ms (60 FPS) steady state, no dropped frames during window animations | Compositor internal frame-timing instrumentation | Release-blocking for animation-touching PRs |
| Browser demo: page load → interactive desktop | ≤ 15s on a reference broadband connection | Synthetic browser benchmark (see [10-TESTING-AND-BUILD.md](10-TESTING-AND-BUILD.md)) | Release-blocking for `web/` changes |
| Package install (small app, warm cache) | ≤ 3s | `novapkg` CLI timer | Tracked, not release-blocking v1 |

## 3. Boot Instrumentation

Each boot milestone in [01-SYSTEM-ARCHITECTURE.md](01-SYSTEM-ARCHITECTURE.md) §2 emits a
timestamp to a fixed-size in-memory ring (no disk I/O on the hot boot path) readable by
Nova Monitor after boot and dumped to the structured log
([01-SYSTEM-ARCHITECTURE.md](01-SYSTEM-ARCHITECTURE.md) §5) once logging is available.
The boot animation's progress states are driven directly by these milestones (see
[01-SYSTEM-ARCHITECTURE.md](01-SYSTEM-ARCHITECTURE.md) §2) — one source of truth for both
the UX and the metric.

## 4. RAM Accounting

The process table in [01-SYSTEM-ARCHITECTURE.md](01-SYSTEM-ARCHITECTURE.md) §3 is the
budget's ledger: each "Always resident" process carries a documented expected RSS range,
reviewed whenever a PR changes what's resident at idle. CI fails a build that pushes
measured idle RAM over budget without an accompanying ADR justifying the increase
(mirroring how [ADR-0002](decisions/ADR-0002-init-and-service-supervision.md) already
treats "add a new daemon" as ADR-worthy).

## 5. Rendering & Animation Performance

- All compositor and Nova UI animations are driven by the token-defined duration/easing
  scale ([06-DESIGN-SYSTEM.md](06-DESIGN-SYSTEM.md) §3), which also makes frame-budget
  auditing tractable — a fixed, known set of motion curves rather than arbitrary
  per-widget timing code.
- GPU path is the default; the software-rendering fallback
  ([04-APPLICATION-FRAMEWORK-AND-SDK.md](04-APPLICATION-FRAMEWORK-AND-SDK.md) §5) carries
  its own, looser frame-time budget (30 FPS floor) since it targets constrained
  environments (old hardware, browser demo) where 60 FPS GPU-tier performance isn't the
  goal — correctness and responsiveness are.

## 6. Application Launch & Runtime

- `nova-sessiond` sandbox construction ([ADR-0010](decisions/ADR-0010-app-sandboxing-model.md))
  is the dominant cost in app launch latency; its steps (namespace setup, seccomp filter
  install, mount setup) are profiled individually so a regression is attributable to a
  specific step, not just "launch got slower."
- Idle (backgrounded, not focused) apps are suspended via
  `App::on_suspend` ([04-APPLICATION-FRAMEWORK-AND-SDK.md](04-APPLICATION-FRAMEWORK-AND-SDK.md)
  §2) with an expectation that suspended apps' CPU usage drops to ~0 — enforced by cgroup
  CPU accounting surfaced in Nova Monitor.

## 7. Disk & Package Performance

- Package install time budget (§2) is dominated by network download + SquashFS mount,
  not decompression-heavy formats — informs compression-level choice in
  `.novapkg` build tooling (favor fast decompression over maximal ratio, tunable per
  release if the tradeoff needs revisiting).
- OS update image size and A/B disk overhead are tracked against
  [ADR-0008](decisions/ADR-0008-filesystem-and-update-strategy.md)'s accepted 2x-disk
  tradeoff — reviewed at each milestone that measurably grows the root image.

## 8. Continuous Monitoring

Every reference-hardware/VM budget in §2 is re-measured on every merge to the main
integration branch (not just at release time) so regressions are caught within a day,
not discovered at release-candidate time — detailed CI mechanics in
[10-TESTING-AND-BUILD.md](10-TESTING-AND-BUILD.md).
