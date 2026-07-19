# NovaOS on Render

Deploys the real Tiny Core Linux desktop (Xorg + `flwm` + `wbar` + `aterm` +
`uzdoom`/Doom) as a container that runs QEMU internally and exposes its display over
the browser via noVNC — so a URL, not a local VM, is how people actually use NovaOS.

## How it works

1. `Dockerfile` builds a Debian-slim image with `qemu-system-x86`, `novnc`, and
   `websockify` installed.
2. `deploy/build-tinycore.sh` runs at `docker build` time (on Render's build
   infrastructure, not your machine): downloads Tiny Core's stock kernel and base
   rootfs, resolves and downloads a curated package set (Xorg-7.7 + the VESA driver +
   Xprogs + `flwm` + `wbar` + `aterm` + `uzdoom` and their transitive deps), merges
   everything into one `novaos-initrd.gz`.
3. `deploy/entrypoint.sh` runs at container start: boots that image under QEMU with
   `-display vnc=0.0.0.0:0` (no `-enable-kvm` — Render doesn't expose hardware
   virtualization, so this runs under QEMU's software CPU emulation, not
   accelerated), waits for the VNC port, then runs `websockify` to bridge it to
   Render's `$PORT` over HTTP, serving noVNC's web client.
4. Visit the deployed URL's `/vnc.html` and the desktop is right there in the
   browser — `wbar`'s taskbar has a Terminal launcher and a Doom launcher.

## About the Doom inclusion — a real risk worth knowing

Tiny Core's package repo only offers one Doom engine: `uzdoom`
(UZDoom, a GZDoom-family source port), pulling in `freedoom` for the actual game data
(free, open, legally redistributable WAD files — not id Software's original assets).
There is **no lightweight classic software-renderer port available** in Tiny Core's
repo (no chocolate-doom, prboom, dsda-doom, etc.) — `uzdoom` is what exists, so it's
what's included.

**The real tension**: `uzdoom`'s own package notes recommend `Xorg-7.7-3d` and ideally
Vulkan for "best graphics load." This deployment runs Xorg with the plain `vesa`
driver (a dumb framebuffer, no OpenGL/DRI acceleration) under QEMU's software CPU
emulation (no GPU passthrough at all, no KVM). GZDoom-family engines do support a
software rendering path, but historically still expect *some* working OpenGL context
to initialize display output even in software-render mode — untested here whether
`uzdoom` degrades gracefully or fails to start entirely in this environment. If it
doesn't work, the fix is either bundling Mesa's `llvmpipe` (software OpenGL — same
approach already proven for `nova-compositor` locally, see project memory) so a real
(software) GL context exists for `uzdoom` to attach to, or accepting that Doom isn't
achievable within Tiny Core's actual package set as shipped and dropping/replacing it.
**Verify this on the first real build — don't assume "it's Doom, it'll obviously
run."**

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
  script, and now includes `uzdoom` which wasn't part of the earlier proven set).
- Whether `uzdoom` actually starts at all given the software-rendering risk above.
- Actual boot time and responsiveness under QEMU's TCG (software) emulation with no
  KVM — will be slower than the KVM-accelerated local testing done earlier, and Doom
  specifically will stress this further.
- RAM: defaults to 1024MB (`QEMU_RAM` env var, bumped up from the original 512MB
  once Doom was added) — still untested whether that's actually enough; Render's own
  container memory limit (set by the plan) needs to comfortably exceed this too.

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
