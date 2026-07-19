#!/bin/bash
set -e

PORT="${PORT:-8080}"
QEMU_RAM="${QEMU_RAM:-1024}"

echo "=== NovaOS: starting QEMU (software emulation, no KVM on Render) ==="
qemu-system-x86_64 \
  -m "$QEMU_RAM" -smp 1 \
  -kernel /opt/novaos/vmlinuz64 \
  -initrd /opt/novaos/novaos-initrd.gz \
  -vga std \
  -display vnc=0.0.0.0:0 \
  -serial mon:stdio \
  -append "console=ttyS0 noembed" &

QEMU_PID=$!
echo "QEMU pid=$QEMU_PID, VNC on :5900"

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
