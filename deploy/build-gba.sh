#!/bin/bash
# Runs during `docker build`, as its own layer after the Game Boy build. No
# Game Boy Advance emulator exists as a Tiny Core package, so this compiles
# VBA-M (VisualBoyAdvance-M) from source, SDL frontend only (no wxWidgets -
# confirmed live that ENABLE_WX=OFF still produces a fully working `vbam`
# binary, so there's no reason to pull in a GUI toolkit dependency for it).
#
# This exists specifically because SameBoy (the Game Boy/Color emulator
# built in the previous stage) is architecturally incapable of running GBA
# games - different CPU, different everything - confirmed the hard way
# after building it and only then discovering the ROM a user actually wanted
# to run (Pokemon FireRed) is GBA, not GB/GBC. Also confirmed the more
# obvious choice, mGBA, has a genuine unresolved threading bug in this
# specific container environment (its core emulation thread never
# completes its startup handshake with the render thread, so nothing ever
# gets presented to the window - reproduced across both its master branch
# and latest stable release, with GL and software rendering, and after
# ruling out a general pthread/environment problem with an isolated test).
# VBA-M's SDL frontend uses a different, simpler threading model and works
# correctly here, confirmed by actually running the ROM in question.
#
# No game ROM is bundled here or ever will be - that's Nintendo's
# copyrighted data, not something this project can legally ship. This only
# builds the emulator itself; a player supplies their own legally-obtained
# ROM file afterward via the file manager.
set -euo pipefail

OUT=/opt/novaos/tc-root

echo "== chroot needs DNS to clone from GitHub =="
cp /etc/resolv.conf "$OUT/etc/resolv.conf"

echo "== clone VBA-M source =="
chroot "$OUT" /usr/bin/env LD_LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib \
  PATH=/usr/local/bin:/usr/local/sbin:/bin:/sbin:/usr/bin:/usr/sbin HOME=/root \
  git clone --depth 1 https://github.com/visualboyadvance-m/visualboyadvance-m.git /root/vbam-src

echo "== configure (SDL frontend only, no wxWidgets GUI, no vcpkg) =="
chroot "$OUT" /usr/bin/env LD_LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib \
  PATH=/usr/local/bin:/usr/local/sbin:/bin:/sbin:/usr/bin:/usr/sbin HOME=/root \
  /bin/sh -c "
    set -e
    cd /root/vbam-src
    mkdir -p build && cd build
    cmake .. \
      -DCMAKE_BUILD_TYPE=Release \
      -DENABLE_SDL=ON \
      -DENABLE_WX=OFF \
      -DENABLE_LINK=OFF \
      -DENABLE_FFMPEG=OFF \
      -DENABLE_ONLINEUPDATES=OFF \
      -DVCPKG_BINARY_PACKAGES=OFF \
      -DUPSTREAM_RELEASE=OFF
  "

echo "== build (this clones a googletest submodule for the test targets - the" \
     "chroot's DNS setup above is what makes that work) =="
NPROC=$(nproc)
chroot "$OUT" /usr/bin/env LD_LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib \
  PATH=/usr/local/bin:/usr/local/sbin:/bin:/sbin:/usr/bin:/usr/sbin HOME=/root \
  /bin/sh -c "cd /root/vbam-src/build && make vbam -j${NPROC}"

echo "== install =="
cp "$OUT/root/vbam-src/build/vbam" "$OUT/usr/local/bin/vbam"
chroot "$OUT" chmod +x /usr/local/bin/vbam
mkdir -p "$OUT/usr/local/share/pixmaps"
cp "$OUT/root/vbam-src/src/wx/icons/sizes/128x128/apps/visualboyadvance-m.png" "$OUT/usr/local/share/pixmaps/vbam.png" 2>/dev/null \
  || echo "  (no bundled icon at the expected path - menu entry will use a fallback icon)"

echo "== clean up build tree (keep only the installed result) =="
chroot "$OUT" rm -rf /root/vbam-src
echo "== VBA-M build complete =="
