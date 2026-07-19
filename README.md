# NovaOS

**NovaOS is Tiny Core Linux, booted, configured, and curated as a real, complete
desktop operating system.** Not a from-scratch desktop platform anymore — see
[docs/00-VISION.md](docs/00-VISION.md) for the full direction change and why.

**Status (2026-07-19)**: real, proven. A modified Tiny Core boot image (custom kernel
modules + a curated real X11/window-manager desktop stack — Xorg with the VESA driver,
`flwm`, `wbar`, `aterm`) boots under QEMU/KVM into a genuine, working desktop: real
window manager decorations, a live terminal with a real shell, a taskbar. Confirmed by
screenshot, not by design document. A separate, earlier proof also exists of
`nova-compositor` (a from-scratch Wayland compositor) booting bare-metal via DRM/KMS —
see [docs/00-VISION.md](docs/00-VISION.md) §1 and §7 for why that path was set aside
in favor of Tiny Core's real, already-complete desktop.

**Not yet in this repository**: the actual build process (downloading Tiny Core's
kernel source, compiling the DRM/GPU modules it doesn't ship by default, resolving and
merging the X11/flwm/wbar/aterm package tree, packing the initramfs, and the QEMU
boot-test/screenshot loop) currently lives in an external WSL2 workspace, not checked
into this repo. Bringing that into `system/` as real, reproducible tooling is the
immediate next step — see §"What's Next" below.

## Read This First

- [docs/00-VISION.md](docs/00-VISION.md) — what NovaOS is now, why the direction
  changed, and what happens to the prior work
- The rest of `docs/` (`docs/01-*` through `docs/14-*`, `docs/specs/`, `docs/rfcs/`) —
  **historical record of the prior, superseded direction** (a from-scratch desktop
  platform: custom compositor, UI toolkit, SDK, package format, app suite). Real,
  working code was built under that plan — see "What's Already Built" below — but it
  is not the active direction. Read `docs/00-VISION.md` before treating anything else
  in `docs/` as current.

## What's Already Built (prior direction, kept, not active)

Real, compiled, tested code exists from the earlier from-scratch-desktop-platform
direction. None of it is deleted; none of it is the current plan.

- `services/nova-bus`, `services/nova-bus-broker` — a real Protobuf-based IPC bus with
  a working broker, proven via `tests/vertical-slice` as separate OS processes.
- `sdk/nova-app`, `sdk/nova-ui`, `sdk/nova-ui-wayland` — app lifecycle, widget, and
  real-Wayland-rendering primitives.
- `desktop/compositor` (`nova-compositor`) — a real Smithay-based Wayland compositor,
  including a from-scratch DRM/KMS bare-metal backend proven to boot and render under
  QEMU/KVM with no host compositor.
- `desktop/shell` (`nova-shell`), `apps/hello-gui`, `apps/nova-files` — a taskbar and
  two real apps, proven running live through `nova-compositor`.
- `tools/nova-bus-bench` — real measured IPC latency/throughput numbers.

See [docs/IMPLEMENTATION-NOTES/](docs/IMPLEMENTATION-NOTES/) for the detailed record of
this work, including one real security bug found and fixed.

## Repository Layout

```
system/    Base system, boot, init, image build, updates — where the Tiny Core
           build/boot tooling belongs once it's brought into the repo (not yet done)
services/  Nova Bus IPC — prior direction, kept, not active
desktop/   nova-compositor + nova-shell — prior direction, kept, not active
sdk/       Nova SDK — prior direction, kept, not active
apps/      hello-gui, nova-files — prior direction, kept, not active
web/       novaos.dev browser demo — prior direction, deferred indefinitely
tools/     Build/CI/dev tooling
tests/     Cross-crate integration and system tests
docs/      Vision (current) + full prior-direction architecture/specs/RFCs (historical)
```

## Building & Testing (prior-direction code)

The Rust workspace above still builds and tests, if you want to run or extend the
prior-direction code:

```sh
cargo build --workspace
cargo test --workspace
cargo run --release -p nova-bus-bench
```

Requires Rust (stable) and `protoc` on `PATH` (or set the `PROTOC` env var).
`desktop/compositor`, `desktop/shell`, `sdk/nova-ui-wayland`, and `apps/hello-gui` are
Linux/Wayland-only and are not workspace default-members — build them explicitly with
`cargo build -p <name>` from a Linux environment (WSL2 with WSLg works for nested
testing; real QEMU/KVM for the bare-metal DRM backend).

This does **not** build or boot the actual NovaOS product anymore — that's the Tiny
Core image described above, built outside this repo for now.

## What's Next

1. Bring the Tiny Core build/boot process (kernel module compilation, package
   resolution/merge, initramfs packing, QEMU boot-test) into this repo under `system/`
   as real, reproducible, checked-in tooling — currently it only exists as ad-hoc
   scripts in an external WSL2 workspace.
2. Decide and record (as an ADR) what curation NovaOS actually wants on top of Tiny
   Core's real desktop — which window manager, which apps, what branding/theming —
   rather than defaulting to whatever was fastest to get booting first (`flwm` +
   `wbar` + `aterm`, chosen because they matched an existing reference screenshot).
3. Boot on real hardware, not just QEMU/KVM.

## Contributing

Not yet open for broad external contribution. See [CONTRIBUTING.md](CONTRIBUTING.md)
for setup and process. Every non-trivial architectural change requires an RFC and/or
ADR before implementation ([docs/rfcs/README.md](docs/rfcs/README.md),
[docs/11-CODING-STANDARDS.md](docs/11-CODING-STANDARDS.md) §8) — including this
direction change, which should get its own ADR (not yet written).

## License

TBD — recorded as an open decision; a permissive/copyleft choice will be captured as an
ADR before this repo accepts external contributions.
