#!/bin/bash
# Runs during `docker build`, as a separate layer after the main Tiny Core
# rootfs is assembled. No pre-built Minetest package exists in Tiny Core's
# repo, so this compiles it from source, inside a chroot into that rootfs -
# using its own gcc/cmake toolchain so the result actually links against the
# same libc/libs as the rest of the system (building with the outer Debian
# layer's toolchain would produce a binary that doesn't run in the TC rootfs).
set -euo pipefail

OUT=/opt/novaos/tc-root
TCZ_URL=http://tinycorelinux.net/16.x/x86_64/tcz
WORK=/build-mt

mkdir -p "$WORK/tcz" "$WORK/extract"
cd "$WORK"

echo "== resolve Minetest build dependencies =="
SEED="cmake gcc glibc_base-dev curl-dev freetype-dev gettext-dev gmp-dev jsoncpp-dev libGL-dev \
  libjpeg-turbo-dev libogg-dev libpng-dev libvorbis-dev luajit-dev mesa-dev \
  sqlite3-dev zlib_base-dev ncursesw-dev openal openal-dev libXi-dev libXrandr-dev libXxf86vm-dev sdl2-dev \
  linux-6.12_api_headers git"
> queue.txt
> resolved.txt
mkdir -p deps
for p in $SEED; do echo "$p" >> queue.txt; done

ROUND=0
while [ -s queue.txt ]; do
  ROUND=$((ROUND+1))
  sort -u queue.txt | comm -23 - <(sort -u resolved.txt) > queue_new.txt
  mv queue_new.txt queue.txt
  [ -s queue.txt ] || break
  cat queue.txt >> resolved.txt
  sort -u resolved.txt -o resolved.txt
  echo "  round $ROUND: resolving $(wc -l < queue.txt) new packages"

  xargs -P 20 -I{} sh -c \
    'f="tcz/{}.tcz"; [ -f "$f" ] || curl -sSf -o "$f" "'"$TCZ_URL"'/{}.tcz" 2>/dev/null || true' \
    < queue.txt
  xargs -P 20 -I{} sh -c \
    'curl -sSf "'"$TCZ_URL"'/{}.tcz.dep" 2>/dev/null > "deps/{}" || true' \
    < queue.txt

  > next_queue.txt
  while read -r pkg; do
    [ -z "$pkg" ] && continue
    [ -s "deps/$pkg" ] || continue
    for d in $(cat "deps/$pkg"); do
      echo "${d%.tcz}" >> next_queue.txt
    done
  done < queue.txt
  sort -u next_queue.txt | comm -23 - <(sort -u resolved.txt) > queue.txt
  [ "$ROUND" -gt 15 ] && { echo "too many rounds, stopping"; break; }
done
echo "== resolved $(wc -l < resolved.txt) build-dependency packages =="

echo "== extract + merge build deps into the rootfs (build-time only, see cleanup below) =="
ls tcz/*.tcz | xargs -P 8 -I{} sh -c '
  f="{}"; name=$(basename "$f" .tcz); dest="extract/$name"
  mkdir -p "$dest"
  unsquashfs -f -d "$dest" "$f" > /dev/null 2>&1 || echo "WARN: failed to extract $name"
'
for d in extract/*/; do
  cp -a "$d." "$OUT/" 2>/dev/null || true
done

echo "== chroot needs DNS to clone from GitHub =="
cp /etc/resolv.conf "$OUT/etc/resolv.conf"

echo "== clone Minetest source (client + bundled minetest_game) =="
chroot "$OUT" /usr/bin/env LD_LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib \
  PATH=/usr/local/bin:/usr/local/sbin:/bin:/sbin:/usr/bin:/usr/sbin HOME=/root \
  git clone --depth 1 --recursive https://github.com/minetest/minetest.git /root/minetest-src

echo "== configure + build Minetest (this is the slow part - full C++ compile) =="
NPROC=$(nproc)
chroot "$OUT" /usr/bin/env LD_LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib \
  PATH=/usr/local/bin:/usr/local/sbin:/bin:/sbin:/usr/bin:/usr/sbin HOME=/root \
  /bin/sh -c "
    set -e
    cd /root/minetest-src
    mkdir -p build && cd build
    cmake .. \
      -DCMAKE_BUILD_TYPE=Release \
      -DRUN_IN_PLACE=FALSE \
      -DCMAKE_INSTALL_PREFIX=/usr/local \
      -DENABLE_SOUND=TRUE \
      -DENABLE_CURL=TRUE \
      -DBUILD_CLIENT=TRUE \
      -DBUILD_SERVER=FALSE
    make -j${NPROC}
    make install
  "

echo "== clean up build tree (keep only the installed result) =="
chroot "$OUT" rm -rf /root/minetest-src
rm -rf "$WORK"
echo "== Minetest build complete =="
