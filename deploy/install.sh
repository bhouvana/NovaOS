#!/bin/sh
# NovaOS one-command installer for Linux and macOS.
#
#   curl -fsSL https://raw.githubusercontent.com/bhouvana/NovaOS/master/deploy/install.sh | bash
#
# Detects whether Docker is installed, installs it if not (fully unattended
# on Linux; macOS needs one manual step Apple doesn't allow scripts to skip -
# see below), pulls the prebuilt NovaOS image, runs it, and opens your
# browser to the desktop. No Docker knowledge required.
set -e

IMAGE="ghcr.io/bhouvana/novaos:latest"
PORT="${NOVAOS_PORT:-8080}"
NAME="novaos"

info()  { printf '\033[1;36m==>\033[0m %s\n' "$1"; }
ok()    { printf '\033[1;32m✓\033[0m %s\n' "$1"; }
warn()  { printf '\033[1;33m!\033[0m %s\n' "$1"; }
die()   { printf '\033[1;31mx\033[0m %s\n' "$1"; exit 1; }

OS="$(uname -s)"

# --- 1. make sure Docker is installed and running --------------------------
if ! command -v docker >/dev/null 2>&1; then
  warn "Docker isn't installed - installing it now."
  case "$OS" in
    Linux)
      info "Running Docker's official install script (this is unattended, no reboot needed)..."
      curl -fsSL https://get.docker.com | sh
      # Start the daemon; on most distros this is systemd, but fall back
      # gracefully if it isn't (e.g. minimal containers, WSL without systemd).
      (sudo systemctl enable --now docker 2>/dev/null) || (sudo service docker start 2>/dev/null) || true
      # Avoid forcing a logout/login just to use docker without sudo: run
      # the rest of this script's docker commands with sudo if needed.
      if ! docker info >/dev/null 2>&1; then
        DOCKER="sudo docker"
      fi
      ;;
    Darwin)
      warn "macOS requires Docker Desktop, which Apple's security model doesn't allow any script to fully install unattended (it's a signed .app that needs one manual launch + permission grant)."
      if command -v brew >/dev/null 2>&1; then
        info "Installing via Homebrew..."
        brew install --cask docker
      else
        info "Opening the Docker Desktop download page - install it, then run this command again."
        open "https://www.docker.com/products/docker-desktop/" 2>/dev/null || true
        die "Install Docker Desktop from the page that just opened, launch it once, then re-run this command."
      fi
      info "Docker Desktop is installed but needs to be launched once manually."
      open -a Docker 2>/dev/null || true
      info "Waiting for Docker Desktop to finish starting (this can take a minute the first time)..."
      i=0
      until docker info >/dev/null 2>&1; do
        i=$((i+1))
        [ "$i" -gt 90 ] && die "Docker Desktop hasn't started after 3 minutes. Open it manually from Applications, wait for the whale icon to settle, then re-run this command."
        sleep 2
      done
      ;;
    *)
      die "Unrecognized OS ($OS). This installer supports Linux and macOS - on Windows, use install.ps1 instead."
      ;;
  esac
  ok "Docker is installed."
else
  ok "Docker is already installed."
fi

DOCKER="${DOCKER:-docker}"

if ! $DOCKER info >/dev/null 2>&1; then
  die "Docker is installed but not running. Start Docker (Docker Desktop on macOS, or 'sudo systemctl start docker' on Linux) and re-run this command."
fi
ok "Docker is running."

# --- 2. get the image --------------------------------------------------------
info "Pulling the NovaOS image (this is a one-time download, a few GB)..."
if ! $DOCKER pull "$IMAGE" 2>/dev/null; then
  warn "Couldn't pull the prebuilt image (not published yet, or offline) - building it locally instead. This is much slower (~20-30 minutes, one time only) since it compiles the whole desktop from source."
  TMPDIR="$(mktemp -d)"
  trap 'rm -rf "$TMPDIR"' EXIT
  command -v git >/dev/null 2>&1 || die "git is required to build locally. Install git and re-run, or wait for the prebuilt image to be published."
  git clone --depth 1 https://github.com/bhouvana/NovaOS.git "$TMPDIR/NovaOS"
  (cd "$TMPDIR/NovaOS" && $DOCKER build -t novaos .)
  IMAGE="novaos"
fi
ok "NovaOS image ready."

# --- 3. run it ----------------------------------------------------------------
# Fail fast on a likely port conflict rather than waiting through a futile
# health-check loop first - 8080 is a very common default for other dev
# tools (Jenkins, Tomcat, etc. all default there too). Best-effort only
# (needs bash's /dev/tcp - silently skipped under a plain POSIX sh, though
# this script is meant to be piped to bash) - the real failure is still
# caught properly by the docker-run exit code check below either way.
if (exec 3<>"/dev/tcp/127.0.0.1/$PORT") 2>/dev/null; then
  exec 3<&- 2>/dev/null
  exec 3>&- 2>/dev/null
  die "Port $PORT is already in use by something else on this machine. Set a different port and re-run, e.g.: NOVAOS_PORT=8081 curl -fsSL https://raw.githubusercontent.com/bhouvana/NovaOS/master/deploy/install.sh | bash"
fi

# The two named volumes are what make this a real second OS instead of a demo
# that forgets everything on restart: your files/settings and anything
# installed via the in-desktop Software Center persist across restarts and
# even NovaOS image updates (everything else always comes from the image, so
# a newer NovaOS pull still gets you the update, not a frozen copy).
$DOCKER rm -f "$NAME" >/dev/null 2>&1 || true
info "Starting NovaOS..."
if ! $DOCKER run -d --name "$NAME" --restart unless-stopped -p "$PORT:8080" -e PORT=8080 --privileged \
  -v novaos-home:/opt/novaos/tc-root/root \
  -v novaos-tce:/opt/novaos/tc-root/etc/sysconfig/tcedir \
  "$IMAGE" >/dev/null; then
  die "NovaOS failed to start. Run 'docker logs $NAME' for details - a port conflict on $PORT is the most likely cause even after the check above, if something grabbed it in between."
fi

info "Waiting for the desktop to come up..."
i=0
until curl -fsS "http://localhost:$PORT/" >/dev/null 2>&1; do
  i=$((i+1))
  [ "$i" -gt 60 ] && die "NovaOS container is running but the desktop never came up at http://localhost:$PORT/. Run 'docker logs $NAME' to see what's wrong."
  sleep 1
done

URL="http://localhost:$PORT/"
ok "NovaOS is running at $URL"
case "$OS" in
  Darwin) open "$URL" 2>/dev/null || true ;;
  Linux)  xdg-open "$URL" 2>/dev/null || true ;;
esac
echo
echo "If your browser didn't open automatically, go to: $URL"
