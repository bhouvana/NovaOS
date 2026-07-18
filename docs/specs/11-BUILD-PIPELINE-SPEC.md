# Spec 11 — Build Pipeline Specification

Status: Draft v0.1 · Last updated: 2026-07-18

Concretizes [../10-TESTING-AND-BUILD.md](../10-TESTING-AND-BUILD.md) §1 into the full
artifact-production chain.

## 1. End-to-End Pipeline

```text
Rust source (services/, desktop/, sdk/, apps/)
   ↓  cargo build --workspace --release
Per-crate compiled artifacts (one binary per service/app,
   one .rlib per SDK crate)
   ↓  (SDK crates only) cargo publish --registry nova-internal
sdk/* crates available to apps/* as ordinary Cargo dependencies
   (internal registry mirror, 04-APPLICATION-FRAMEWORK-AND-SDK.md §3)
   ↓  (apps/* only) novapkg-builder <app-dir> <manifest.toml>
   (07-PACKAGE-FORMAT-SPEC.md §7)
.novapkg per app: SquashFS payload + header + Ed25519 signature
   ↓
   ├──→ published to Nova Store catalog (apps only reach end users
   │     this way — never bundled directly into the OS image except
   │     as pre-installed defaults, see §2)
   │
   └──→ (if a default/pre-installed app) fed into the image builder
system/image/ recipes (base packages list, service binary paths,
   pre-installed .novapkg list, kernel config)
   ↓  tools/image-builder assemble --profile=<full-iso|browser-demo>
   (11-BUILD-PIPELINE-SPEC.md §2)
SquashFS root filesystem (base system + services + compositor +
   shell + pre-installed apps' mounted .novapkg content)
   ↓
   ├──→ --profile=full-iso: system/image/ partitions into A/B slots
   │     + EFI/GRUB → novaos-<version>.iso
   │     (ADR-0008, 05-PACKAGE-AND-UPDATE-SYSTEM.md)
   │
   └──→ --profile=browser-demo: trimmed app set, software-rendering
         default, v86-detection kernel cmdline flag set
         (08-BROWSER-ARCHITECTURE-SPEC.md §7)
         → nova-browser-demo.img
   ↓
Both artifacts signed (system image signing key, distinct from
   per-publisher .novapkg signing keys — 08-SECURITY-MODEL.md §4)
   ↓
   ├──→ novaos-<version>.iso → Downloads page (13-WEBSITE-INFORMATION-
   │     ARCHITECTURE.md) + update channel manifest
   │     (05-PACKAGE-AND-UPDATE-SYSTEM.md §5)
   │
   └──→ nova-browser-demo.img → web/ build → deployed to novaos.dev
         CDN alongside the React site bundle (11-BUILD-PIPELINE-SPEC
         §3)
```

## 2. Image Builder Profiles

`tools/image-builder` (a `tools/` crate/script per
[../02-REPOSITORY-STRUCTURE.md](../02-REPOSITORY-STRUCTURE.md)) takes one required
`--profile` flag; both profiles share the same `system/image/` recipe *inputs*, differing
only in a profile-specific manifest listing which pre-installed apps and kernel config
options apply — this is the mechanism behind
[../07-BROWSER-DEPLOYMENT.md](../07-BROWSER-DEPLOYMENT.md) §3's "one source tree, two
targets, never a maintained fork" claim, made concrete:

```toml
# system/image/profiles/full-iso.toml
kernel_config = "full"              # broad driver/firmware set
preinstalled_apps = ["files", "terminal", "notes", "paint", "calculator",
                      "monitor", "package-center", "browser",
                      "arcade-chess", "arcade-snake", "arcade-sudoku",
                      "arcade-minesweeper", "arcade-solitaire"]
rendering_default = "gpu"
partition_scheme = "a-b-efi"

# system/image/profiles/browser-demo.toml
kernel_config = "minimal-virtio"    # v86-relevant drivers only
preinstalled_apps = ["files", "terminal", "arcade-snake"]
rendering_default = "software"
partition_scheme = "single"          # no A/B needed — demo state lives in
                                      # v86 IndexedDB snapshots, not an
                                      # updatable installed system
cmdline_extra = "nova.v86_profile=1"  # detected by nova-browser-bridge,
                                       # 08-BROWSER-ARCHITECTURE-SPEC §7
```

## 3. CI Integration

Maps onto the 8 stages in
[../10-TESTING-AND-BUILD.md](../10-TESTING-AND-BUILD.md) §2:

| CI stage | Pipeline step exercised |
|---|---|
| 1–3 (fast checks, unit, contract) | `cargo build`/`test` only — no image assembly, runs on every push |
| 4 (integration/boot test) | Full `--profile=full-iso` build + headless QEMU boot, release-branch gated |
| 5 (performance regression) | Same image, boots against [02-MEMORY-BUDGET.md](02-MEMORY-BUDGET.md)/[03-BOOT-TIMELINE.md](03-BOOT-TIMELINE.md) thresholds |
| 8 (browser demo smoke test) | `--profile=browser-demo` build + headless-browser v86 boot test |

## 4. Reproducibility

Both profiles pin: exact Rust toolchain version (`rust-toolchain.toml` at workspace
root), exact Alpine package versions
([ADR-0001](../decisions/ADR-0001-linux-base-distribution.md)) via a lockfile-style
manifest in `system/image/`, and exact `novapkg-builder`/`tools/image-builder` tool
versions — a given commit hash + `--profile` value is expected to produce a
byte-identical artifact modulo embedded build timestamps (stripped from the reproducible
build path per [../10-TESTING-AND-BUILD.md](../10-TESTING-AND-BUILD.md) §1), required for
the update system's signature-based trust model
([../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §4) to be independently auditable.

## 5. Local Developer Loop

```text
cargo build -p nova-compositor          # fast, single-crate iteration
   ↓
tools/dev-vm run --incremental          # boots the dev VM's *existing*
                                          # image with just the changed
                                          # binary hot-swapped in, not a
                                          # full image rebuild
                                          (10-TESTING-AND-BUILD.md §5)
```

A full `tools/image-builder assemble` run is reserved for pre-PR validation and CI, not
the inner dev loop — the incremental hot-swap path is what keeps
[../00-VISION.md](../00-VISION.md) §3's developer-friendliness goal true day-to-day.
