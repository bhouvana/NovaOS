# NovaOS — Repository Structure

Status: Draft v0.1 · Owner: Chief Architect · Last updated: 2026-07-18

## 1. Goals

- Every top-level folder has exactly one owner (in the "who reviews PRs here" sense) and
  one clear boundary matching a box in
  [01-SYSTEM-ARCHITECTURE.md](01-SYSTEM-ARCHITECTURE.md) §1.
- Shared code (SDK, protocols) is physically separated from consumers (apps) so the
  dependency direction is enforced by folder structure, not just by convention.
  See §3, Dependency Rules.
- A newcomer can guess where a given piece of functionality lives without asking.

## 2. Top-Level Layout

```
NovaOS/
├── docs/                     Architecture docs, ADRs, roadmap (this tree)
│   └── decisions/            ADRs
├── system/                   Base system: kernel config, initramfs, OpenRC scripts,
│                              boot animation, A/B update tooling, image build recipes
│   ├── boot/                 Bootloader config, Nova boot animation client
│   ├── init/                 OpenRC service scripts for Nova Services
│   ├── image/                Image build (SquashFS root, A/B partition layout)
│   └── update/               update-agent, signature verification
├── services/                 Resident Nova Services (one crate per service)
│   ├── nova-bus/             IPC broker (novabusd) + wire protocol (.proto schemas)
│   ├── nova-sessiond/        Session manager, app lifecycle, sandboxing
│   ├── nova-themed/          Theme engine
│   ├── novapkg/              Package manager (agent + CLI)
│   └── permission-broker/    Permission grant/prompt logic (used by nova-sessiond)
├── desktop/                  Desktop shell: compositor + shell surfaces
│   ├── compositor/           nova-compositor (wlroots-based)
│   ├── shell/                Taskbar, Launcher, Notification Center (nova-shell)
│   └── settings/             Nova Settings app (shell-level, ships with the OS)
├── sdk/                      Nova SDK — the only thing third-party app authors depend on
│   ├── nova-ui/               UI toolkit (widgets, layout, theming, a11y)
│   ├── nova-app/              App entrypoint, window/lifecycle API
│   ├── nova-storage/          Per-app storage API
│   ├── nova-notify/           Notifications client API
│   ├── nova-settings-api/     Settings read/subscribe API
│   ├── nova-clipboard/        Clipboard + drag-and-drop API
│   └── nova-plugin/           Plugin/extension API and scripting host
├── apps/                     First-party Nova apps (each consumes sdk/ only, never
│                              services/ or desktop/ internals directly)
│   ├── nova-files/
│   ├── nova-terminal/
│   ├── nova-notes/
│   ├── nova-paint/
│   ├── nova-calculator/
│   ├── nova-monitor/
│   ├── nova-package-center/
│   ├── nova-browser/
│   └── nova-arcade/           Chess, Snake, Sudoku, Minesweeper, Solitaire as sub-crates
│       ├── chess/
│       ├── snake/
│       ├── sudoku/
│       ├── minesweeper/
│       └── solitaire/
├── web/                       novaos.dev site: v86 integration, ISO/image hosting glue,
│                              docs site
├── tools/                     Dev tooling: CI scripts, VM launch scripts, ISO builder
│                              CLI, lint configs, ADR/doc linting
├── tests/                     Cross-crate integration/E2E tests (boot tests, SDK
│                              contract tests) — unit tests live beside their crates
├── Cargo.toml                 Workspace root (all Rust crates under one workspace)
├── CHANGELOG.md
├── CONTRIBUTING.md
└── README.md
```

## 3. Dependency Rules

Enforced by CI (dependency-graph lint, see
[10-TESTING-AND-BUILD.md](10-TESTING-AND-BUILD.md)), not just documented:

1. `apps/*` → may depend on `sdk/*` only. Never on `services/*` or `desktop/*` crates
   directly, and never on another app's crate.
2. `sdk/*` → may depend on `services/nova-bus`'s generated client stubs (the wire
   protocol) but never on a service's internal implementation crate.
3. `desktop/*` → may depend on `services/*` and `sdk/nova-ui` (the shell is itself built
   with Nova UI, eating its own dog food) but apps never depend on `desktop/*`.
4. `services/*` → may depend on each other only in the direction listed in
   [01-SYSTEM-ARCHITECTURE.md](01-SYSTEM-ARCHITECTURE.md) §3's process table (e.g.
   `nova-sessiond` depends on `nova-bus`, not the reverse).
5. `system/*` → depends on nothing above it; it is the base the rest boots on.
6. Nothing depends on `web/`; `web/` depends only on build artifacts (the ISO/image),
   never on source crates directly.

Rationale: this mirrors the layering in
[01-SYSTEM-ARCHITECTURE.md](01-SYSTEM-ARCHITECTURE.md) §1 exactly, so "is this
architecturally correct" reduces to "does the folder path match the layer."

## 4. Ownership Model

Each top-level folder carries a `CODEOWNERS`-style entry (added once the project has more
than one contributor) and a short `README.md` stating: purpose, current status, and a
link back to the relevant architecture doc(s). Placeholder READMEs are seeded now (see
§5) so the mapping exists from day one, before implementation.

## 5. Status

This structure is created as an empty skeleton in Phase 1 (see
[12-ROADMAP-AND-MILESTONES.md](12-ROADMAP-AND-MILESTONES.md)) — folders and README
placeholders only, no implementation code, so that Phase 2+ work has an agreed home for
everything before the first line of Rust is written.
