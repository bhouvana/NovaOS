# NovaOS — Development Roadmap & Milestones

Status: Draft v0.1 · Owner: Chief Architect · Last updated: 2026-07-18

## 1. Sequencing Principle

Each phase is independently testable and produces a real, runnable artifact — never a
multi-phase "big bang" integration. A phase does not start until the previous phase's
exit criteria are met, though research/design work for the next phase may begin early.

## 2. Phase 1 — Research, Architecture, Repository (complete)

**Scope**: everything in this `docs/` tree, plus the empty, structured repository
skeleton ([02-REPOSITORY-STRUCTURE.md](02-REPOSITORY-STRUCTURE.md)).

**Deliverables**:
- Vision, system architecture, subsystem architecture docs (this tree).
- 10 foundational ADRs (`docs/decisions/`).
- Repository skeleton with placeholder READMEs, no implementation code.
- Coding standards, testing/build strategy, roadmap (this doc).

**Exit criteria**: every doc in this tree exists and cross-references resolve; repository
skeleton matches [02-REPOSITORY-STRUCTURE.md](02-REPOSITORY-STRUCTURE.md) exactly.

## 3. Phase 1.5 — Engineering Specification (complete, pending final confirmation)

**Scope**: Phase 1 answered *what* to build and *why*. Phase 1.5 answers *how the pieces
interact*, in enough detail that Phase 2 is transcription, not design-while-coding. No
Rust is written in this phase. Full spec set lives in
[docs/specs/](specs/00-INDEX.md).

**Why this phase exists**: added after Staff Engineer review of the Phase 1 output —
layer diagrams and ADRs are necessary but not sufficient to hand a subsystem to an
engineer without them re-deriving dozens of decisions Phase 1 left implicit (exact
message sequences, exact byte layouts, exact numeric budgets, exact widget APIs). Every
one of those re-derivations is a rework risk if two engineers guess differently.

**Deliverables** (see [docs/specs/00-INDEX.md](specs/00-INDEX.md) for the full set and
status of each):
- Component interaction / sequence diagrams for every cross-subsystem flow.
- A fully itemized memory budget (every resident consumer accounted for, not just a
  target range).
- A millisecond-level boot timeline for both reference VM and reference hardware.
- A window manager specification: lifecycle, states, focus model, z-order, damage
  tracking, frame scheduling.
- A Nova UI toolkit specification: widget tree, layout algorithm, rendering pipeline,
  animation system, theming API.
- A Nova SDK specification: full app model, manifest schema, and every API surface
  (storage, notifications, settings, clipboard/DnD, localization).
- A byte-level `.novapkg` package format specification.
- A full browser-architecture specification (novaos.dev stack, input/clipboard/
  persistence/networking bridging into the emulated guest).
- Per-application design docs for every first-party app.
- A design bible with concrete, final numeric values (not just token *names*).
- A build-pipeline specification (source → artifact, every stage).
- A browser-demo experience specification (the actual onboarding UX at novaos.dev).
- A website information architecture (full site map, not just "download page").
- An ecosystem-vision document reconciling long-term platform ambitions (Store, Cloud,
  Sync, Community) with the no-cloud-account-required non-goal.

**Second review round (2026-07-18)**: a Staff Engineer pass on the first Phase 1.5
output found it ~90–95% complete but flagged several foundational contracts as still
too expensive to leave implicit before code exists. Closed with:
- **RFCs for every core service** — [docs/rfcs/](rfcs/README.md), one per resident/
  logical service (Nova Shell, Nova Bus, Nova WM, Package Service, Notification Service,
  Theme Service, Settings Service, Session Manager, Update Service), each covering
  purpose, dependencies, public APIs, events, config, startup order, failure modes,
  recovery, metrics, logging, and security considerations.
- **A complete Nova Bus wire protocol spec** — [docs/specs/15-NOVA-BUS-PROTOCOL-SPEC.md](specs/15-NOVA-BUS-PROTOCOL-SPEC.md).
- **A consolidated state-machine reference** — [docs/specs/16-STATE-MACHINES.md](specs/16-STATE-MACHINES.md).
- **An SDK public API stability policy** — [docs/specs/17-SDK-API-REFERENCE-POLICY.md](specs/17-SDK-API-REFERENCE-POLICY.md).
- **A full plugin architecture spec** — [docs/specs/18-PLUGIN-ARCHITECTURE-SPEC.md](specs/18-PLUGIN-ARCHITECTURE-SPEC.md).
- **A complete filesystem layout spec** — [docs/specs/19-FILESYSTEM-LAYOUT-SPEC.md](specs/19-FILESYSTEM-LAYOUT-SPEC.md).
- **A configuration strategy spec** (TOML, decided and justified) — [docs/specs/20-CONFIGURATION-STRATEGY-SPEC.md](specs/20-CONFIGURATION-STRATEGY-SPEC.md).
- **An observability spec** (logs, metrics, tracing, debug overlays, health checks) — [docs/specs/21-OBSERVABILITY-SPEC.md](specs/21-OBSERVABILITY-SPEC.md).
- **A release process spec** (channels, signing hierarchy, rollback, changelogs) — [docs/specs/22-RELEASE-PROCESS-SPEC.md](specs/22-RELEASE-PROCESS-SPEC.md).
- **An engineering principles capstone doc** — [docs/ENGINEERING-PRINCIPLES.md](../ENGINEERING-PRINCIPLES.md).
- **A hardened, structural (not conventional) rendering boundary** between the
  novaos.dev React site and NovaOS's own desktop UI —
  [docs/specs/08-BROWSER-ARCHITECTURE-SPEC.md](specs/08-BROWSER-ARCHITECTURE-SPEC.md) §1b.
- **RFC process adoption as a standing condition of entering implementation** — every
  non-trivial architectural change (new subsystem, protocol change, SDK change,
  packaging change) requires an RFC and/or ADR before implementation, formalized in
  [docs/rfcs/README.md](rfcs/README.md) and restated in
  [11-CODING-STANDARDS.md](11-CODING-STANDARDS.md) §8.

**Exit criteria**: every doc in `docs/specs/` and `docs/rfcs/` exists, is internally
consistent with Phase 1's ADRs and architecture docs (no contradictions), every numeric
budget in Phase 1 (RAM, boot time) is fully itemized rather than stated as a range with
no accounting, and every core resident/logical service has an Accepted RFC. Phase 2
does not start until this phase is signed off — as of the second review round above,
this phase is considered complete pending final confirmation.

## 4. Phase 2 — Vertical Slice (current phase)

**Scope**: redefined from the original "Infrastructure & Skeletons" framing after
Staff Engineer review of the completed Phase 1.5 spec set. The review's core point:
building every subsystem a little bit (horizontal) risks discovering integration
problems only after all of them are partially built. Building one complete path through
every layer (vertical) proves the architecture actually holds together while the
codebase is still small enough to change cheaply.

**The slice**:

```text
Boot → Linux starts → OpenRC → Nova Shell → Nova Bus → Nova WM
   → Nova UI Toolkit → Desktop → Launcher → click "Hello"
   → App starts → Window appears → IPC works → Window closes
```

Every architectural layer in [01-SYSTEM-ARCHITECTURE.md](01-SYSTEM-ARCHITECTURE.md) §1
gets exercised by this one path. No fake implementations, no shortcuts — a component
that isn't real yet is simply not part of the slice, rather than being stubbed out and
declared "done."

**Explicitly excluded from Phase 2** (deferred to Phase 4+ regardless of how tempting):
Nova Browser, Nova Arcade, Nova Paint, Nova Notes, Nova Settings (beyond the minimum
needed to prove the theme/settings *path* exists), theming beyond a single default,
window animations, multi-monitor. These all depend on the foundation this phase
validates — building them before the foundation is proven is exactly the horizontal
expansion this phase exists to avoid.

**Exit criteria** (strict — every item below must be independently demonstrated, not
just implemented):

| Area | Must demonstrate |
|---|---|
| Boot | Linux boots; OpenRC starts Nova services in the documented order ([specs/16-STATE-MACHINES.md](specs/16-STATE-MACHINES.md) §5); structured logging works ([specs/21-OBSERVABILITY-SPEC.md](specs/21-OBSERVABILITY-SPEC.md) §1) |
| Nova Bus | Services register; messages send; replies work; timeouts work; error paths work — the full contract in [specs/15-NOVA-BUS-PROTOCOL-SPEC.md](specs/15-NOVA-BUS-PROTOCOL-SPEC.md) exercised, not just the happy path |
| Nova WM | One window: drag, resize, close, focus — the minimum slice of [specs/04-WINDOW-MANAGER-SPEC.md](specs/04-WINDOW-MANAGER-SPEC.md) needed for a usable single window |
| Nova UI Toolkit | Label, Button, layout, mouse, keyboard — the minimum slice of [specs/05-NOVA-UI-TOOLKIT-SPEC.md](specs/05-NOVA-UI-TOOLKIT-SPEC.md) needed to build the Hello app below |
| SDK | A real Hello World app (`Window::new("Hello Nova")`) compiles, launches, runs, closes, using the real `App` trait ([specs/06-NOVA-SDK-SPEC.md](specs/06-NOVA-SDK-SPEC.md) §1–§2) |
| Package System | Install one package, launch it, uninstall it — [specs/16-STATE-MACHINES.md](specs/16-STATE-MACHINES.md) §2's states each actually observed, not assumed |
| Browser Demo | `localhost` (or the packaged demo image) shows a desktop appearing — not polished, functional only |

**Environment note**: this phase's compositor/boot/browser-demo rows require a Linux
graphics toolchain (wlroots, a bootable kernel image, QEMU) not present in every
development environment this project has been worked in so far. Where that toolchain is
unavailable, the Nova Bus, SDK, and Package System rows are provable today in pure Rust
with real cross-process tests; the Nova WM/UI/Boot/Browser-Demo rows are tracked as
blocked-on-environment until a Linux build environment is available, rather than being
marked done on the strength of unit tests alone — an exit criterion is only met by the
demonstration it names, never by a proxy for it.

## 4a. Phase 2.5 — System Validation

**Scope**: inserted between the vertical slice and broad feature development,
per Staff Engineer review — the cheapest point in the project to change course is
immediately after the first real code exists and before a second subsystem is built on
top of assumptions the slice may have falsified. Renamed from "Architecture Validation"
after the vertical slice landed: by this point there's implementation, an API surface,
and measured data to validate, not just the architecture in the abstract — the name
should say what's actually being checked.

**What this phase actually is**: not a deliverable-producing phase, but a structured
review pass asking uncomfortable questions while the codebase is still small enough
that the answer "we got this wrong" is cheap to act on. Four categories:

**Architecture** — did the design hold up under real use?
- Is Nova Bus actually the right abstraction, now that real services use it —
  did the request/response + pub/sub split ([ADR-0006](decisions/ADR-0006-ipc-mechanism.md))
  hold up, or did the Hello World flow want something the protocol doesn't cleanly
  express? (The Response-spoofing gap found and fixed during the vertical slice —
  [IMPLEMENTATION-NOTES/0004](IMPLEMENTATION-NOTES/0004-nova-bus-response-authorization.md)
  — is exactly this kind of finding, already closed rather than deferred to this phase.)
- Is the compositor API too complex for what Phase 3's window-management policy work
  will need, based on how much code the vertical slice's minimal drag/resize/close/focus
  slice actually took?
- Are there unnecessary abstractions — should three services be one, or one be three,
  based on how the real dependency graph turned out to behave rather than how it looked
  on paper?

**Ergonomics** — is the API pleasant to use, not just correct?
- Can someone build an app without reading 40 pages of spec first? If the answer is no,
  the SDK changes, not the newcomer's onboarding checklist.
- Is the API too verbose for what it does — e.g. does `Window::new(...)` need to be
  `WindowBuilder::new(...)`, or does the builder pattern exist somewhere out of habit
  rather than need?
- Did writing the real Hello World app
  ([specs/06-NOVA-SDK-SPEC.md](specs/06-NOVA-SDK-SPEC.md)) feel like the API was designed
  for this use case, or did it fight the app author at any point?

**Performance** — measured, not estimated. See
[IMPLEMENTATION-NOTES/](IMPLEMENTATION-NOTES/) and `tools/nova-bus-bench` for the first
real numbers gathered during Phase 2 itself (Nova Bus call latency p50/p95/p99/max,
throughput, timeout rate; app-startup stage timing) — this phase's job is to decide
whether those numbers are good enough, not to gather them for the first time.
- Is the memory budget ([specs/02-MEMORY-BUDGET.md](specs/02-MEMORY-BUDGET.md)) realistic
  now that real (if platform-caveated — see the IMPLEMENTATION-NOTES entry on this)
  process memory numbers exist for `novabusd` and one running app?
- Are startup times meeting the [specs/03-BOOT-TIMELINE.md](specs/03-BOOT-TIMELINE.md)
  budget, measured for real rather than estimated, for the portions measurable without
  a compositor?
- Does IPC throughput and latency leave enough headroom for the message volume Phase 3's
  real window-manager traffic (focus changes, damage events) will add?

**Process** — does the documentation trail actually work?
- Are [IMPLEMENTATION-NOTES/](IMPLEMENTATION-NOTES/) entries getting written *as*
  reality diverges from the spec, or only reconstructed after the fact under review
  pressure? (If the latter, that's a process finding about this phase itself.)

**Output**: a short written retro (not a new doc category — an addendum appended to this
roadmap doc, dated) recording what held up, what didn't, and which RFCs/ADRs need
amendment as a result, per each document's own changelog/revisit-trigger mechanism
([decisions/README.md](decisions/README.md), [rfcs/README.md](rfcs/README.md)). Phase 3
does not start until this retro is written, even though — unlike Phase 1/1.5/2 — this
phase has no CI-checkable exit criterion of its own; the discipline is procedural, not
mechanical.

## 5. Phase 3 — Core Desktop

**Scope**: the desktop shell becomes usable.

**Deliverables**:
- `nova-compositor`: window management policy (floating + snapping), decorations,
  animations, multi-monitor/HiDPI ([03-DESKTOP-ARCHITECTURE.md](03-DESKTOP-ARCHITECTURE.md) §2).
- `nova-shell`: Launcher, Taskbar, Notification Center.
- `nova-sessiond`: app lifecycle + sandboxing ([ADR-0010](decisions/ADR-0010-app-sandboxing-model.md)).
- `nova-themed` + Nova Light/Dark themes ([06-DESIGN-SYSTEM.md](06-DESIGN-SYSTEM.md) §6).
- Nova Settings (basic: display, theme, network passthrough).
- Boot animation wired to real milestones ([09-PERFORMANCE-STRATEGY.md](09-PERFORMANCE-STRATEGY.md) §3).

**Exit criteria**: boot to a usable, themed desktop; launch/switch/close a sandboxed
placeholder app via the Launcher and Taskbar; meets the idle-RAM and boot-time budgets
([09-PERFORMANCE-STRATEGY.md](09-PERFORMANCE-STRATEGY.md) §2).

## 6. Phase 4 — Core Applications

**Scope**: the SDK matures under real app pressure; first real apps ship.

**Deliverables**: Nova Terminal, Nova Files, Nova Monitor, Nova Notes, Nova Calculator,
Nova Package Center (client only — Nova Store backend stood up in parallel),
`novapkg` CLI ([05-PACKAGE-AND-UPDATE-SYSTEM.md](05-PACKAGE-AND-UPDATE-SYSTEM.md)).

**Exit criteria**: install/update/remove a real app through Package Center end-to-end;
each app meets its [09-PERFORMANCE-STRATEGY.md](09-PERFORMANCE-STRATEGY.md) §2 launch
budget; SDK reaches its first `1.0.0` (semver committed,
[11-CODING-STANDARDS.md](11-CODING-STANDARDS.md) §10).

## 7. Phase 5 — Browser, Games, Developer SDK

**Scope**: the remaining first-party surface, plus opening the platform to third parties.

**Deliverables**: Nova Browser (embedded engine, [ADR-0005](decisions/ADR-0005-ui-toolkit.md)
Consequences), Nova Arcade (Chess, Snake, Sudoku, Minesweeper, Solitaire), `nova-cli new`
scaffolding tool, published SDK documentation site content
([04-APPLICATION-FRAMEWORK-AND-SDK.md](04-APPLICATION-FRAMEWORK-AND-SDK.md) §8).

**Exit criteria**: a sample third-party-style app, built by someone outside the core team
using only public SDK docs, runs correctly — the real test of "is the SDK actually
usable," not just "does it compile in-tree."

## 8. Phase 6 — Browser Demo, Installer, Recovery, Release

**Scope**: ship v1.0.

**Deliverables**: novaos.dev site + v86 integration
([07-BROWSER-DEPLOYMENT.md](07-BROWSER-DEPLOYMENT.md)), installer (real-hardware install
flow, disk partitioning, A/B setup), recovery mode (boot into previous A/B slot / minimal
recovery shell if both slots fail health checks), full documentation site, v1.0 release
per [11-CODING-STANDARDS.md](11-CODING-STANDARDS.md) §12.

**Exit criteria**: all of [00-VISION.md](00-VISION.md) §5's success criteria are met and
independently verifiable (published RAM/boot numbers, installable ISO, working browser
demo, documented SDK with an external sample app).

## 9. Milestone Gate Summary

| Phase | Primary Risk If Skipped | Gate |
|---|---|---|
| 1 | Architecture drift, rework | Docs complete & cross-referenced |
| 1.5 | Engineers re-derive interaction/format details independently, diverge, rework | `docs/specs/` complete & internally consistent |
| 2 | Building every subsystem partway (horizontal) instead of proving one full path (vertical); building on unproven foundations | Every row in Phase 2's exit-criteria table independently demonstrated |
| 2.5 | Broad feature work amplifies a foundational mistake before anyone asks whether the foundation is right | Written retro appended to this doc; RFC/ADR amendments filed for anything the slice falsified |
| 3 | Unusable shell blocking app work | Themed desktop meets perf budget |
| 4 | SDK designed in a vacuum | Real app ships through real Package Center |
| 5 | SDK not actually usable externally | External sample app succeeds |
| 6 | Shipping without proof of success criteria | All v1.0 criteria independently verified |

Each phase's exit criteria is the input to the next phase's Phase 1-style design review —
large architectural changes discovered mid-phase get an ADR, not a silent scope change.
