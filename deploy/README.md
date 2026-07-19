# NovaOS on Render

Deploys the real Tiny Core Linux desktop (Xorg + `flwm` + `wbar` + `aterm`) as a
container that runs QEMU internally and exposes its display over the browser via
noVNC — so a URL, not a local VM, is how people actually use NovaOS.

## How it works

1. `Dockerfile` builds a Debian-slim image with `qemu-system-x86`, `novnc`, and
   `websockify` installed.
2. `deploy/build-tinycore.sh` runs at `docker build` time (on Render's build
   infrastructure, not your machine): downloads Tiny Core's stock kernel and base
   rootfs, resolves and downloads the same curated X11/flwm/wbar/aterm package set
   proven working earlier (Xorg-7.7 + the VESA driver + Xprogs + flwm + wbar + aterm
   and their transitive deps), merges everything into one `novaos-initrd.gz`.
3. `deploy/entrypoint.sh` runs at container start: boots that image under QEMU with
   `-display vnc=0.0.0.0:0` (no `-enable-kvm` — Render doesn't expose hardware
   virtualization, so this runs under QEMU's software CPU emulation, not
   accelerated), waits for the VNC port, then runs `websockify` to bridge it to
   Render's `$PORT` over HTTP, serving noVNC's web client.
4. Visit the deployed URL's `/vnc.html` and the desktop is right there in the
   browser.

## Status: written, not yet build-tested

This was written and reasoned through carefully, but not actually run end-to-end —
the local Docker engine (which shares WSL2's disk) was unavailable at write time due
to an unrelated disk-space incident (see project memory / session history). Things
worth verifying on the first real build:

- Debian bookworm's `qemu-system-x86`, `novnc`, `websockify` package names/paths
  (assumed `novnc`'s web root is `/usr/share/novnc` — standard for the Debian
  package, but unverified here).
- `build-tinycore.sh`'s package resolution against the *live* Tiny Core repo (same
  logic proven working earlier this project, just not re-run against this exact
  script).
- Actual boot time and responsiveness under QEMU's TCG (software) emulation with no
  KVM — will be slower than the KVM-accelerated local testing done earlier.
- RAM: defaults to 512MB (`QEMU_RAM` env var) — untested whether that's enough
  headroom for Xorg + flwm + wbar + aterm under emulation; Render's own container
  memory limit (set by the plan) needs to comfortably exceed this too.

## Deploying

```sh
git push   # to whatever repo Render is connected to
```

Or via `render.yaml` (included) for infra-as-code deploy — see Render's docs for
`render blueprint` / dashboard "New from render.yaml" flow.

## Local testing (once Docker is available)

```sh
docker build -t novaos .
docker run -p 8080:8080 -e PORT=8080 novaos
# open http://localhost:8080/vnc.html
```
