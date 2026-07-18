# Spec 02 — Memory Budget

Status: Draft v0.1 · Last updated: 2026-07-18

Itemizes the target stated in [../00-VISION.md](../00-VISION.md) §5 and
[../09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md) §2. Measured as RSS on
the reference VM (2 vCPU, KVM, virtio-gpu) at "desktop shown, zero user apps open."
Numbers are engineering estimates to be replaced with measured values starting Phase 2
Exit Criteria; each row becomes a named threshold in the CI regression gate
([../09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md) §4).

## 1. Minimum Boot Configuration (floor of the range)

| Component | RSS | Notes |
|---|---|---|
| Linux kernel (resident, post-boot: core + loaded modules) | 12 MB | Kernel text is not swappable; modules limited to what reference hardware profile requires (GPU/net/storage drivers only) |
| musl libc + shared library baseline (libdrm, libwayland, etc., counted once — shared pages) | 3 MB | Shared across all Nova processes; counted once per [../01-SYSTEM-ARCHITECTURE.md](../01-SYSTEM-ARCHITECTURE.md) convention |
| OpenRC + udev/mdev (post-boot resident portion) | 3 MB | OpenRC itself exits after reaching the runlevel; mdev event daemon remains |
| `novabusd` (Nova Bus broker) | 2 MB | Rust binary, minimal — no client connections beyond a handful of idle sockets |
| `nova-sessiond` | 3 MB | Session/sandbox state for zero running apps; grows ~0.3 MB per tracked app |
| `nova-themed` | 1 MB | Small: token set is a few KB in memory, most of this is Rust runtime baseline |
| `permission-broker` | 1 MB | Mostly idle; grant store is a small in-memory table |
| `nova-compositor` (wlroots + GPU driver context, zero client surfaces) | 10 MB | Includes DRM/KMS buffers for the framebuffer's own scanout, wlroots' internal state |
| `nova-shell` (Taskbar + Launcher + Notification Center, one process) | 8 MB | Includes its own Nova UI instance: font atlas, icon atlas, widget tree for 3 surfaces |
| Output framebuffers (compositor scanout + 1 back buffer, 1080p, double-buffered) | 4 MB | `1920×1080×4 bytes × 2 ≈ 16.6 MB` in theory; budgeted at 4 MB reflecting compressed/tiled GPU memory accounting on the RAM side (VRAM-backed on discrete/most integrated GPUs is not counted against system RSS) — reference hardware without a capable GPU uses the software-rendering fallback ([../04-APPLICATION-FRAMEWORK-AND-SDK.md](../04-APPLICATION-FRAMEWORK-AND-SDK.md) §5), tracked as a separate profile in §3 |
| Kernel networking/VFS slab (idle) | 2 MB | Baseline slab allocator usage with no active connections |
| **Subtotal (fixed floor)** | **49 MB** | |
| Filesystem page cache (reclaimable, warms up serving the above binaries + assets) | 15 MB | Reclaimable under memory pressure — the kernel treats this as available, not truly "used" |
| **Floor total** | **64 MB** | Matches the stated floor in [../00-VISION.md](../00-VISION.md) §5 |

## 2. Typical Steady-State Desktop (top of the range)

Same fixed floor (49 MB) plus:

| Component | RSS | Notes |
|---|---|---|
| Filesystem page cache (steady use: font/icon assets, recently-read app binaries) | 25 MB | Grows with usage, still reclaimable |
| Nova Settings (pinned, commonly left open) | 5 MB | One representative always-around app |
| `novapkg-agent` (woken for a catalog check, not exited yet) | 3 MB | On-demand per [../01-SYSTEM-ARCHITECTURE.md](../01-SYSTEM-ARCHITECTURE.md) §3, but commonly recently-active |
| Headroom / kernel accounting slack | 4 MB | Buffer against measurement noise, not allocated to a specific feature |
| **Typical total** | **~100 MB** | Matches the stated ceiling in [../00-VISION.md](../00-VISION.md) §5 |

If a genuinely idle desktop (no apps, no recent Package Center activity) still measures
above ~70 MB once implemented, that is a regression against this budget, not "still
within the range" — the range's upper bound accounts for light real use, not baseline
idle.

## 3. Software-Rendering Profile (low-end hardware / browser demo)

Applies when GPU-accelerated rendering is unavailable
([../04-APPLICATION-FRAMEWORK-AND-SDK.md](../04-APPLICATION-FRAMEWORK-AND-SDK.md) §5,
[ADR-0009](../decisions/ADR-0009-browser-boot-emulator.md)):

| Delta from §1 | RSS | Notes |
|---|---|---|
| `nova-compositor` software rasterizer path (replaces GPU context) | +6 MB | CPU-side framebuffer + rasterizer working set, no GPU driver context to offset it |
| Output framebuffer (single-buffered, software path — v86 has no vsync/page-flip) | -2 MB vs. §1's 4 MB | Simpler buffering strategy under emulation |
| **Net delta** | **+4 MB** | Floor becomes ~68 MB under this profile — still within the stated 64–100 MB range |

## 4. Per-App Marginal Cost

Not part of the idle budget (apps are on-demand,
[../01-SYSTEM-ARCHITECTURE.md](../01-SYSTEM-ARCHITECTURE.md) §3), but tracked so
Package Center and Nova Monitor can show accurate per-app cost:

| App class | Typical RSS | Notes |
|---|---|---|
| Simple utility (Calculator) | 4–6 MB | Nova UI instance + trivial state |
| Data-heavy (Files, Terminal) | 6–10 MB | Directory listings / scrollback buffers |
| Nova Browser | 15 MB idle, 40–80 MB per active tab | The one app with a heavier embedded engine ([ADR-0005](../decisions/ADR-0005-ui-toolkit.md) Consequences) — excluded from the "cohesive lightweight OS" idle budget by design, since it's user-invoked, not resident |
| Arcade game (Snake, Sudoku, Minesweeper, Solitaire) | 5–8 MB | Nova UI + small game-state model |
| Arcade game (Chess, with an engine) | 10 MB | Includes move-generation/search tables |

## 5. Enforcement

Every row's RSS becomes a named constant in the Phase 2 CI performance-regression suite
([../10-TESTING-AND-BUILD.md](../10-TESTING-AND-BUILD.md) §2 stage 5); a component
exceeding its budgeted row by more than 20% fails the build unless the PR includes an
updated version of this document with rationale (mirrors the ADR-required pattern in
[../09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md) §4).
