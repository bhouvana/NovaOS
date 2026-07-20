# NovaOS

**A complete, fully-loaded desktop operating system that runs in your browser.**
Built on Tiny Core Linux, running at native speed inside a container — no VM, no
install wizard, no dual-boot. One command and it's just... there.

![NovaOS status](https://img.shields.io/badge/status-working-brightgreen)

## Get it running

Never touched Docker before? Doesn't matter — paste one of these into your terminal
and everything (including Docker itself, if you don't have it) gets set up for you.

**macOS / Linux:**
```sh
curl -fsSL https://raw.githubusercontent.com/bhouvana/NovaOS/master/deploy/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/bhouvana/NovaOS/master/deploy/install.ps1 | iex
```

That's it. Your browser opens to `http://localhost:8080` with the full desktop running.

> **One honest caveat**: on a machine that's never had Docker before, Windows and
> macOS both require Docker Desktop, which is a signed installer neither Apple nor
> Microsoft let any script fully automate — it may ask you to finish one manual step
> (launching it once, or a restart if WSL2 wasn't already enabled on Windows). If that
> happens, the script tells you exactly what to do — just run the same command again
> afterward and it picks up right where it left off. Linux needs none of this; Docker
> installs completely unattended there.

## What you get

A real desktop, not a tech demo — right-click for a full categorized app menu, or use
the taskbar at the bottom:

- **Terminal** with a real shell — `git`, `gcc`, `python3`, `vim`, `tmux`, and more
  are all there and usable, not just installed for show.
- **Office & productivity**: LibreOffice, AbiWord, Gnumeric, GIMP, Inkscape, Darktable
- **Internet**: Midori, Dillo, NetSurf, SeaMonkey, Thunderbird, HexChat, FileZilla,
  qBittorrent, PuTTY, Remmina
- **Media**: VLC, mpv, Audacity, Audacious, HandBrake
- **Games**: Doom, SuperTux, Neverball/Neverputt, Luanti (Minetest, compiled from
  source), DOSBox-X, Chess, Minesweeper, Bubble Shooter, and the full solitaire family
- **Software Center**: install anything else live, from Tiny Core's 3,500+ package
  repository, without rebuilding or restarting anything

None of this is a curated demo subset — it's the actual, working thing.

## How it works

No QEMU, no nested virtualization. NovaOS chroots straight into a real Tiny Core
Linux userland assembled at build time, running at native container speed (Docker
containers already share the host kernel — that's the whole trick). `Xvfb` provides
the display, `x11vnc` + `noVNC` bridge it to your browser.

- [`Dockerfile`](Dockerfile) — the build, in three stages: the curated ~400-package
  desktop, Minetest compiled from source (no prebuilt package exists for it), then the
  runtime scripts layered on last so editing them doesn't force a rebuild of the
  expensive stuff.
- [`deploy/build-tinycore.sh`](deploy/build-tinycore.sh) — resolves and merges the
  package set from Tiny Core's live repo, in parallel.
- [`deploy/chroot-start.sh`](deploy/chroot-start.sh) — runs inside the chroot: X
  server, window manager, taskbar, wallpaper, the right-click menu, the Software
  Center.
- [`deploy/entrypoint.sh`](deploy/entrypoint.sh) — the container's actual entry point:
  sets up what device access it can, starts the chroot, bridges to the browser.

Running with `--privileged` (which the install scripts do for you) is what makes the
in-desktop terminal work — it needs a real `devpts` mount, which unprivileged
containers can't provide. Without it, everything else still works, just without a
terminal.

## Running it yourself, manually

If you'd rather not use the install script:

```sh
docker pull ghcr.io/bhouvana/novaos:latest
docker run -d --name novaos --restart unless-stopped -p 8080:8080 -e PORT=8080 --privileged ghcr.io/bhouvana/novaos:latest
# open http://localhost:8080
```

Or build it from source yourself (slower — compiles the whole desktop, including
Minetest, from scratch):

```sh
git clone https://github.com/bhouvana/NovaOS.git
cd NovaOS
docker build -t novaos .
docker run -d --name novaos --restart unless-stopped -p 8080:8080 -e PORT=8080 --privileged novaos
```

## Contributing

Not yet open for contribution.

## License

TBD.
