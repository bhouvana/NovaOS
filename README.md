# NovaOS

**NovaOS is Tiny Core Linux, booted, configured, and curated as a real, complete
desktop operating system — deployed so actual people can use it in a browser, not
just run locally.** See [docs/00-VISION.md](docs/00-VISION.md) for the full direction
and why it changed from an earlier from-scratch-desktop-platform plan.

**The actual goal**: a URL where real people can use NovaOS's desktop, not a local
QEMU VM only its author has seen. See [deploy/](deploy/) for the Render deployment —
a Docker image that builds the Tiny Core desktop at container-build time and serves
it over the browser via QEMU + noVNC.

**Status (2026-07-19)**:
- Proven locally, by screenshot: Tiny Core + Xorg (VESA driver) + `flwm` + `wbar` +
  `aterm` boots under QEMU/KVM into a genuine, working desktop — real window manager
  decorations, a live terminal with a real shell, a taskbar. That build process still
  lives in an external WSL2 workspace, not yet checked into this repo as reproducible
  tooling.
- Written, not yet build-tested: [`Dockerfile`](Dockerfile) +
  [`deploy/`](deploy/) — a self-contained build (runs on Render's build
  infrastructure, doesn't depend on the local WSL2 workspace at all) that fetches
  Tiny Core fresh, merges the curated desktop package set (`flwm` + `wbar` + `aterm` +
  `uzdoom`/Doom), and serves it over noVNC. Local Docker testing was blocked by an
  unrelated host disk-space incident at write time — see
  [deploy/README.md](deploy/README.md) for exactly what's unverified, including a real
  open question about whether Doom's engine will run at all without GPU acceleration.

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
- [Dockerfile](Dockerfile), [deploy/](deploy/) — the Render deployment: builds and
  serves the NovaOS desktop over the browser via QEMU + noVNC.
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## What's Next

1. Actually build-test the Docker image and deploy it to Render — confirm the
   package names, boot time, and RAM budget noted as unverified in
   [deploy/README.md](deploy/README.md).
2. Bring the *local* Tiny Core build/boot process (proven working via WSL2/QEMU,
   used for local iteration and screenshots) into this repo too, separately from the
   Docker deployment path — currently only ad-hoc scripts in an external workspace.
3. Decide and record (as a new ADR) what curation NovaOS actually wants on top of Tiny
   Core's real desktop — which window manager, which apps, what branding/theming —
   rather than defaulting to whatever booted first (`flwm` + `wbar` + `aterm`, chosen
   because it matched an existing reference screenshot).
4. Write an ADR for this direction change itself.
5. Boot on real hardware, not just QEMU/KVM.

## Contributing

Not yet open for contribution.

## License

TBD.
