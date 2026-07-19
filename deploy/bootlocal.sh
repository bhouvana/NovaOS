#!/bin/sh
echo "" > /dev/console
echo "=== NovaOS: booting real Tiny Core X11/FLWM desktop ===" > /dev/console

mkdir -p /root /tmp/.X11-unix
chmod 1777 /tmp/.X11-unix
export HOME=/root
export PATH=/usr/local/bin:/usr/local/sbin:$PATH
export LD_LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib

cp /opt/wbar.conf /root/.wbar 2>/dev/null

echo "=== NovaOS: launching Xorg directly ===" > /dev/console
/usr/local/bin/Xorg :0 vt1 -config /opt/xorg.conf -nolisten tcp -ac -novtswitch > /dev/console 2>&1 &

echo "=== NovaOS: waiting for X socket ===" > /dev/console
for i in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do
  if [ -S /tmp/.X11-unix/X0 ]; then
    echo "=== NovaOS: X socket ready after ${i}s ===" > /dev/console
    break
  fi
  sleep 1
done

export DISPLAY=:0
echo "=== NovaOS: launching flwm ===" > /dev/console
flwm > /dev/console 2>&1 &
sleep 1
xsetroot -solid "#4a6fa5" >> /dev/console 2>&1
echo "=== NovaOS: launching wbar + aterm ===" > /dev/console
wbar -pos bottom -isize 32 > /dev/console 2>&1 &
aterm -geometry 80x24+50+50 > /dev/console 2>&1 &

echo "=== NovaOS: desktop launch sequence complete ===" > /dev/console
