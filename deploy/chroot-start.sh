#!/bin/sh
# Runs inside the chroot'd Tiny Core rootfs - native speed, no VM/kernel emulation.
export HOME=/root
export PATH=/usr/local/bin:/usr/local/sbin:/bin:/sbin:/usr/bin:/usr/sbin
export LD_LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib

mkdir -p /root /tmp/.X11-unix
chmod 1777 /tmp/.X11-unix
cp /opt/wbar.conf /root/.wbar 2>/dev/null

echo "=== NovaOS: starting Xvfb (virtual framebuffer, no VM needed) ==="
# -listen tcp: aterm needs a real PTY, which the chroot can't provide (no
# privilege to mount devpts here) - it runs outside the chroot instead,
# connecting to this X server over TCP rather than the chroot-local socket.
Xvfb :0 -screen 0 800x600x16 +extension GLX +extension RANDR -listen tcp &

echo "=== NovaOS: waiting for X socket ==="
for i in 1 2 3 4 5 6 7 8 9 10; do
  [ -S /tmp/.X11-unix/X0 ] && { echo "X socket ready after ${i}s"; break; }
  sleep 1
done

export DISPLAY=:0
echo "=== NovaOS: launching flwm ==="
flwm &
sleep 1
xsetroot -solid "#4a6fa5"
echo "=== NovaOS: launching wbar ==="
wbar -pos bottom -isize 32 &

echo "=== NovaOS: desktop launch sequence complete, starting x11vnc ==="
exec x11vnc -display :0 -forever -shared -rfbport 5900 -nopw -quiet
