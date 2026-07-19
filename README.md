# NovaOS

**NovaOS is Tiny Core Linux, booted, configured, and curated as a real, complete
desktop operating system.** See [docs/00-VISION.md](docs/00-VISION.md) for the full
direction and why it changed from an earlier from-scratch-desktop-platform plan.

**Status (2026-07-19)**: real, proven, not yet in this repo. A modified Tiny Core boot
image (custom kernel modules + a curated real X11/window-manager desktop stack — Xorg
with the VESA driver, `flwm`, `wbar`, `aterm`) boots under QEMU/KVM into a genuine,
working desktop: real window manager decorations, a live terminal with a real shell, a
taskbar. Confirmed by screenshot, not by design document. The actual build process
(compiling Tiny Core's DRM/GPU kernel modules, resolving and merging the X11 package
tree, packing the initramfs, the QEMU boot-test loop) currently lives in an external
WSL2 workspace — bringing it into this repo as real, reproducible tooling is the
immediate next step.

## What's Here

This repo previously held a from-scratch desktop platform (custom Wayland compositor,
UI toolkit, SDK, app suite, IPC bus, package format) built under an earlier direction.
That direction was abandoned 2026-07-19 in favor of curating Tiny Core Linux's own
real, complete desktop rather than reimplementing one — see
[docs/00-VISION.md](docs/00-VISION.md) §1 for why. The old code and documentation for
that direction were removed from the working tree the same day; they're recoverable
from git history (the commit immediately before the removal) if ever needed again.

What remains:

- [docs/00-VISION.md](docs/00-VISION.md) — the current vision and goals.
- [docs/decisions/](docs/decisions/) — the ADR process, and ADR-0001 (base Linux
  distribution), still accurate: Tiny Core.
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## What's Next

1. Bring the Tiny Core build/boot process (kernel module compilation, `.tcz` package
   resolution/merge, initramfs packing, QEMU boot-test) into this repo — currently it
   only exists as ad-hoc scripts in an external WSL2 workspace, nothing checked in.
2. Decide and record (as a new ADR) what curation NovaOS actually wants on top of Tiny
   Core's real desktop — which window manager, which apps, what branding/theming —
   rather than defaulting to whatever booted first (`flwm` + `wbar` + `aterm`, chosen
   because it matched an existing reference screenshot).
3. Write an ADR for this direction change itself.
4. Boot on real hardware, not just QEMU/KVM.

## Contributing

Not yet open for contribution — there's no buildable tooling in the repo yet to
onboard around (see "What's Next" above).

## License

TBD.
