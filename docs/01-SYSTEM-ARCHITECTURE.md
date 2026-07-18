# NovaOS — System Architecture

Status: Draft v0.1 · Owner: Chief Architect · Last updated: 2026-07-18

Related decisions: [ADR-0001](decisions/ADR-0001-linux-base-distribution.md)–[ADR-0006](decisions/ADR-0006-ipc-mechanism.md)

## 1. Layered View

```
┌─────────────────────────────────────────────────────────────┐
│  Apps            Nova Files · Terminal · Notes · Paint ·     │
│                   Calculator · Monitor · Package Center ·    │
│                   Browser · Arcade (Chess/Snake/Sudoku/...)  │
├─────────────────────────────────────────────────────────────┤
│  SDK              Nova UI toolkit · Window API · Storage ·   │
│                   Notifications API · Settings API ·         │
│                   Clipboard/DnD · Plugin API                 │
├─────────────────────────────────────────────────────────────┤
│  Desktop Shell    Nova Compositor (WM) · Launcher · Taskbar ·│
│                   Notification Center · Settings shell       │
├─────────────────────────────────────────────────────────────┤
│  Nova Services    Session Manager · Nova Bus (IPC) ·         │
│                   novapkg (Package Manager) · Update Agent · │
│                   Theme Engine · Permission Broker           │
├─────────────────────────────────────────────────────────────┤
│  Base System      OpenRC (init) · musl libc · BusyBox tools ·│
│                   udev/mdev · network stack config           │
├─────────────────────────────────────────────────────────────┤
│  Linux Kernel     Upstream Linux (unmodified scheduler/mm/   │
│                   net/drivers)                                │
├─────────────────────────────────────────────────────────────┤
│  Firmware/Boot    UEFI/BIOS → GRUB or systemd-boot-class      │
│                   bootloader → Nova boot animation            │
└─────────────────────────────────────────────────────────────┘
```

Each layer only depends on the layer(s) below it. Apps never talk to the kernel or Nova
Services directly except through the SDK; the SDK never talks to hardware directly except
through the Desktop Shell/Nova Services. This is enforced structurally by the repository
layout (see [02-REPOSITORY-STRUCTURE.md](02-REPOSITORY-STRUCTURE.md) §Dependency Rules),
not just by convention.

## 2. Boot Sequence

```
Firmware (UEFI/BIOS)
   → Bootloader (GRUB, A/B slot selection — ADR-0008)
      → Linux kernel + initramfs
         → OpenRC minimal runlevel (ADR-0002): mount root (read-only
           SquashFS + OverlayFS — ADR-0008), start udev/mdev, network,
           novabusd
            → Nova boot animation starts (own framebuffer/DRM client,
              shown as early as the kernel can hand off a display)
               → Nova Session Manager starts
                  → Nova Compositor starts (ADR-0003), takes display
                     → Desktop Shell starts: Taskbar, Launcher,
                       Notification Center
                        → Ready: boot animation fades, desktop shown
```

Boot-time budget and measurement methodology are defined in
[09-PERFORMANCE-STRATEGY.md](09-PERFORMANCE-STRATEGY.md). The boot animation is not
decorative filler — it is the visible portion of an otherwise-headless sequence and must
be driven by real milestones (kernel handoff, services ready, compositor ready), not a
fixed-duration animation, so it never lies about progress.

## 3. Process Topology (steady state, desktop idle)

| Process | Owner layer | Resident? | Notes |
|---|---|---|---|
| `init` (OpenRC PID 1) | Base System | Always | ADR-0002 |
| `udev`/`mdev` | Base System | Always | device events |
| `novabusd` (Nova Bus broker) | Nova Services | Always | ADR-0006 |
| `nova-sessiond` (Session Manager) | Nova Services | Always | app lifecycle, sandboxing (ADR-0010) |
| `nova-compositor` | Desktop Shell | Always | ADR-0003 |
| `nova-shell` (Taskbar + Launcher + Notification Center) | Desktop Shell | Always | single process, three surfaces |
| `nova-themed` (Theme Engine) | Nova Services | Always | small; theme tokens + live switching |
| `novapkg-agent` | Nova Services | On demand | wakes for install/update/search, not resident |
| `update-agent` | Nova Services | Periodic wake, not resident | checks A/B update channel (ADR-0008) |
| Nova apps (Files, Terminal, …) | Apps | On demand | sandboxed per ADR-0010, exit when closed |
| `Xwayland` | Desktop Shell (optional) | On demand only | ADR-0003, lazy-started |

"Always resident" is a short, deliberately small list — this table is the primary
artifact used to defend the 64–100 MB idle RAM budget; any addition to the "Always" column
requires an ADR justifying the RAM cost.

## 4. Technology Stack Summary

| Concern | Choice | ADR |
|---|---|---|
| Base distro | Alpine Linux (build-time only) | [0001](decisions/ADR-0001-linux-base-distribution.md) |
| Init/supervision | OpenRC + Nova Session Manager | [0002](decisions/ADR-0002-init-and-service-supervision.md) |
| Display/compositor | Wayland via wlroots, custom compositor | [0003](decisions/ADR-0003-compositor-and-display-protocol.md) |
| Language | Rust (C only at FFI boundaries) | [0004](decisions/ADR-0004-systems-language.md) |
| UI toolkit | Nova UI (custom, GPU-accelerated) | [0005](decisions/ADR-0005-ui-toolkit.md) |
| IPC | Nova Bus (Unix sockets + Protobuf) | [0006](decisions/ADR-0006-ipc-mechanism.md) |
| Package format | `.novapkg` (SquashFS + manifest) | [0007](decisions/ADR-0007-package-format.md) |
| Root FS / updates | Read-only SquashFS, A/B, OverlayFS | [0008](decisions/ADR-0008-filesystem-and-update-strategy.md) |
| Browser boot | v86 (WASM x86 emulator) | [0009](decisions/ADR-0009-browser-boot-emulator.md) |
| App sandboxing | namespaces + seccomp + Landlock | [0010](decisions/ADR-0010-app-sandboxing-model.md) |

## 5. Cross-Cutting Concerns

- **Configuration**: a single, versioned TOML-based config tree under `/nova/config`,
  read by Nova Services at startup and hot-reloadable via Nova Bus events where safe
  (theme, keybindings) — no per-app ad hoc config formats.
- **Logging**: structured, leveled logging from every Nova process to a small ring-buffer
  log service (not a full journald-class system — consistent with ADR-0002's rejection of
  systemd's baseline cost), viewable live in Nova Monitor and persisted to
  `/nova/data/logs` with rotation.
- **Session Management**: Nova Session Manager is the single source of truth for "what
  apps are running, in what sandbox, owned by which user session" — see
  [03-DESKTOP-ARCHITECTURE.md](03-DESKTOP-ARCHITECTURE.md) §Session Management.
- **Power Management**: handled via standard kernel/ACPI interfaces surfaced through a
  thin Nova Services wrapper (suspend/resume, battery status over Nova Bus) — no
  reinvented power daemon; where Linux already solves the problem, we thinly wrap it,
  consistent with "we consume upstream Linux, we do not fork it"
  ([00-VISION.md](00-VISION.md) §1).
- **Networking**: standard Linux networking (NetworkManager-class functionality) exposed
  to Nova Settings via Nova Bus; NovaOS does not reimplement Wi-Fi/DHCP/routing logic.
- **Accessibility**: a first-class concern of Nova UI (see
  [ADR-0005](decisions/ADR-0005-ui-toolkit.md) consequences), not a separate subsystem —
  every widget must be accessible by construction.

## 6. What Lives Where (subsystem → repo pointer)

Detailed folder-level ownership is in
[02-REPOSITORY-STRUCTURE.md](02-REPOSITORY-STRUCTURE.md). At the architecture level, each
box in §1's diagram maps 1:1 to a top-level repository area, so "who owns this layer" is
always answerable by looking at the folder tree, not tribal knowledge.
