#!/bin/bash
set -e

PORT="${PORT:-8080}"
TCROOT=/opt/novaos/tc-root

echo "=== NovaOS: creating minimal device nodes for the chroot (build-time mknod is blocked by sandboxed builders like Render's, but CAP_MKNOD is normally available at container runtime) ==="
mkdir -p "$TCROOT/dev/pts"
cd "$TCROOT/dev"
[ -e null ]    || mknod -m 666 null c 1 3    2>/dev/null || echo "  (mknod null failed - continuing)"
[ -e zero ]    || mknod -m 666 zero c 1 5    2>/dev/null || echo "  (mknod zero failed - continuing)"
[ -e random ]  || mknod -m 666 random c 1 8  2>/dev/null || echo "  (mknod random failed - continuing)"
[ -e urandom ] || mknod -m 666 urandom c 1 9 2>/dev/null || echo "  (mknod urandom failed - continuing)"
[ -e tty ]     || mknod -m 666 tty c 5 0     2>/dev/null || echo "  (mknod tty failed - continuing)"
[ -e ptmx ]    || mknod -m 666 ptmx c 5 2    2>/dev/null || echo "  (mknod ptmx failed - continuing)"
cd /

echo "=== NovaOS: best-effort bind mounts for /dev, /proc, /sys (skipped if unprivileged) ==="
mount --bind /dev "$TCROOT/dev" 2>/dev/null || echo "  (no /dev bind - continuing without it)"
mount --bind /dev/pts "$TCROOT/dev/pts" 2>/dev/null || echo "  (no /dev/pts bind - continuing without it)"
mount --bind /proc "$TCROOT/proc" 2>/dev/null || echo "  (no /proc bind - continuing without it)"
mount --bind /sys "$TCROOT/sys" 2>/dev/null || echo "  (no /sys bind - continuing without it)"

echo "=== NovaOS: starting Tiny Core desktop natively via chroot (no VM, no CPU emulation) ==="
chroot "$TCROOT" /opt/chroot-start.sh &

echo "=== NovaOS: waiting for X server TCP port (6000) ==="
for i in $(seq 1 20); do
  if (echo > /dev/tcp/127.0.0.1/6000) 2>/dev/null; then
    echo "X server ready after ${i}s"
    break
  fi
  sleep 1
done

# Known limitation: aterm only supports Unix98 PTY allocation (/dev/ptmx +
# ptsname()), which needs a mounted devpts instance to resolve the slave path.
# Render's containers (like local unprivileged Docker) grant no CAP_SYS_ADMIN,
# so devpts can't be mounted anywhere, in or out of the chroot - confirmed via
# direct testing (mount -t devpts fails with EPERM even locally). Running aterm
# outside the chroot avoids the PTY error but breaks its terminfo/locale/passwd
# lookups (hardcoded to the real root), causing a crash instead. Left disabled
# until a proper fix (e.g. a custom launcher using TIOCGPTPEER, or patching
# aterm) - the rest of the desktop (flwm, wbar, all GUI apps, Doom) is unaffected.
echo "=== NovaOS: terminal app (aterm) skipped - known PTY limitation under unprivileged containers ==="

echo "=== NovaOS: waiting for VNC port ==="
for i in $(seq 1 30); do
  if (echo > /dev/tcp/127.0.0.1/5900) 2>/dev/null; then
    echo "VNC ready after ${i}s"
    break
  fi
  sleep 1
done

echo "=== NovaOS: starting noVNC/websockify on \$PORT=$PORT ==="
exec websockify --web=/usr/share/novnc "0.0.0.0:$PORT" localhost:5900
