#!/bin/bash
# Runs during `docker build`, as its own layer after Minetest. No Game Boy /
# Game Boy Color emulator exists as a Tiny Core package, so this compiles
# SameBoy from source - chosen because it's accurate, actively maintained,
# and (confirmed by testing this exact build live before writing this
# script) builds cleanly against the SDL2/libpng/OpenAL/Mesa dev headers
# already merged into the rootfs by the Minetest stage, so this needs no
# extra build-dependency resolution of its own.
#
# No game ROM is bundled here or ever will be - that's Nintendo's
# copyrighted data, not something this project can legally ship. This only
# builds the emulator itself; a player supplies their own legally-obtained
# ROM file afterward via the file manager.
#
# SameBoy also ships its own boot ROM replacements (its own clean-room code,
# not Nintendo's - a separate, real distinction from the game ROM issue
# above), but building those needs the rgbds assembler, which isn't
# packaged in Tiny Core's repo either. Skipped: confirmed directly that the
# emulator runs fine without them, just without the original boot logo/
# animation - it jumps straight into whatever ROM is loaded instead.
set -euo pipefail

OUT=/opt/novaos/tc-root

echo "== chroot needs DNS to clone from GitHub =="
cp /etc/resolv.conf "$OUT/etc/resolv.conf"

echo "== clone SameBoy source =="
chroot "$OUT" /usr/bin/env LD_LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib \
  PATH=/usr/local/bin:/usr/local/sbin:/bin:/sbin:/usr/bin:/usr/sbin HOME=/root \
  git clone --depth 1 https://github.com/LIJI32/SameBoy.git /root/sameboy-src

echo "== build SameBoy's SDL frontend (targeted at just the binary - the full" \
     "'sdl' make target also wants boot ROMs, which need the rgbds assembler" \
     "this build doesn't have) =="
chroot "$OUT" /usr/bin/env LD_LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib \
  PATH=/usr/local/bin:/usr/local/sbin:/bin:/sbin:/usr/bin:/usr/sbin HOME=/root \
  /bin/sh -c "
    set -e
    cd /root/sameboy-src
    make build/bin/SDL/sameboy CONF=release
  "

echo "== install =="
cp "$OUT/root/sameboy-src/build/bin/SDL/sameboy" "$OUT/usr/local/bin/sameboy"
chroot "$OUT" chmod +x /usr/local/bin/sameboy
mkdir -p "$OUT/usr/local/share/pixmaps"
cp "$OUT/root/sameboy-src/FreeDesktop/AppIcon/128x128.png" "$OUT/usr/local/share/pixmaps/sameboy.png"

echo "== clean up build tree (keep only the installed result) =="
chroot "$OUT" rm -rf /root/sameboy-src
echo "== SameBoy build complete =="
