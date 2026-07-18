# NovaOS — Vision & Product Goals

Status: Draft v0.1 · Owner: Chief Architect · Last updated: 2026-07-18

## 1. What NovaOS Is

**NovaOS is a lightweight desktop platform built on Linux** — that is the framing to use
publicly, deliberately in preference to "a Linux distro." It is not a kernel project, not
a driver project, and not a repackaging of an existing desktop environment. It is a new
user- and developer-facing OS layer — desktop, window manager, applications, SDK, package
system, and update mechanism — that happens to use Linux as its foundation, the way
ChromeOS, SteamOS, and Android do. The distinction matters because it correctly signals
where the engineering investment actually goes: the desktop, the developer platform, and
the user experience, not the kernel underneath it, which we consume unmodified
(§ below).

**Non-goals**: replacing the Linux scheduler, memory manager, network stack, or drivers.
We consume upstream Linux; we do not fork it.

## 2. Why NovaOS Exists

Existing options force a tradeoff NovaOS refuses to accept:

| Option | Problem |
|---|---|
| Mainstream desktop Linux distros (Ubuntu, Fedora) | Assembled from independently-designed components (GNOME/KDE + GTK/Qt + systemd + a package manager none of which were co-designed) → inconsistent UX, heavy idle RAM, slow boot |
| ChromeOS | Cohesive and fast, but closed, cloud-dependent, not developer-extensible in the way we want |
| SteamOS | Cohesive and fast, but purpose-built for gaming, not a general desktop/dev platform |
| Tiny Core / Puppy Linux | Tiny and fast, but dated UX, minimal app ecosystem, not "production quality" |

NovaOS's bet: it is possible to get ChromeOS-grade cohesion and boot speed, SteamOS-grade
performance discipline, and a real desktop-OS application/SDK ecosystem, in one system,
if every component is designed together instead of assembled from unrelated projects.

## 3. Product Goals (ranked)

1. **Cohesion** — every visible surface (boot animation, desktop, apps, settings) looks and
   behaves like one product, not a collection of Linux utilities.
2. **Low resource footprint** — idle RAM 64–100 MB (see [ADR-0001](decisions/ADR-0001-linux-base-distribution.md),
   [09-PERFORMANCE-STRATEGY.md](09-PERFORMANCE-STRATEGY.md)); runs on decade-old hardware.
3. **Fast, animated, branded boot** — firmware → kernel → Nova, in seconds, not tens of
   seconds.
4. **Instant accessibility** — boots in a browser tab at novaos.dev with zero install,
   running the *real* OS image, not a mockup.
5. **Developer-friendly** — a documented SDK, a real package format, a real build system,
   contribution docs from day one.
6. **Maintainable at scale** — architected to support 100,000+ LOC without becoming
   unmaintainable; every subsystem has one clear owner and one clear boundary.
7. **Open source** — permissive contribution model, public ADRs, public roadmap.

## 4. Target Environments

- Real hardware: x86_64 first (broadest driver coverage via upstream Linux), aarch64 as a
  stretch goal.
- Virtual machines: QEMU/KVM, VirtualBox, VMware — primary development and CI target.
- Browser: WASM-compiled x86 emulator booting the real NovaOS ISO (see
  [07-BROWSER-DEPLOYMENT.md](07-BROWSER-DEPLOYMENT.md)).

## 5. Success Criteria (v1.0 definition of done)

A v1.0 release is a bootable ISO plus a working novaos.dev browser demo that together
demonstrate:

- Desktop shell: compositor/WM, launcher, taskbar, notifications, settings, theming
  (light/dark).
- Native app suite: Files, Terminal, Text Editor, Paint, Calculator, System Monitor,
  Package Center, Browser.
- Nova Arcade: Chess, Snake, Sudoku, Minesweeper, Solitaire.
- Package manager with signed packages and a working install/update/remove flow.
- SDK with documented APIs (windowing, UI toolkit, storage, notifications, clipboard,
  drag-and-drop, settings) and at least one third-party-style sample app built against it,
  not shipped in-tree.
- Idle RAM within the 64–100 MB budget, measured and published.
- Boot time budget met and published (see [09-PERFORMANCE-STRATEGY.md](09-PERFORMANCE-STRATEGY.md)).
- Installable to real hardware and bootable in-browser from the same ISO artifact.

## 6. Design Philosophy

Every architectural decision is evaluated against, in order: **simplicity,
maintainability, consistency, performance, developer experience, low memory, beautiful
UX, modularity.** When two of these conflict, the earlier one in this list wins unless an
ADR explicitly justifies otherwise.

Concretely this means: prefer one well-integrated component over three loosely-integrated
ones; prefer no daemon over a daemon; prefer a boring, well-understood technology over a
novel one unless the novel one earns its complexity with a measured benefit; prefer
deleting a feature over half-finishing it.

## 7. Non-Goals (explicit)

- Not a general-purpose server OS.
- Not a from-scratch kernel, bootloader, or driver stack.
- Not binary-compatible with every existing Linux desktop app out of the box (compatible
  where cheap — e.g., via a compatibility app-runner — but not a design constraint that
  shapes the core architecture).
- Not cloud-account-gated. Local accounts work fully offline.
- Telemetry and crash reporting are opt-in, off by default, and scoped narrowly (see
  [08-SECURITY-MODEL.md](08-SECURITY-MODEL.md)).
