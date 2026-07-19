#!/bin/bash
# Runs during `docker build`. Fetches Tiny Core's stock kernel + base rootfs,
# resolves and downloads a curated X11/flwm/wbar/aterm desktop package set,
# merges everything into a single initramfs NovaOS boots from at runtime.
set -euo pipefail

WORK=/build
BASE_URL=http://tinycorelinux.net/16.x/x86_64
TCZ_URL="$BASE_URL/tcz"
OUT=/opt/novaos

mkdir -p "$WORK/rootfs" "$WORK/tcz" "$WORK/extract" "$OUT"
cd "$WORK"

echo "== fetching stock kernel + base rootfs =="
curl -sSf -o vmlinuz64 "$BASE_URL/release/distribution_files/vmlinuz64"
curl -sSf -o corepure64.gz "$BASE_URL/release/distribution_files/corepure64.gz"
cp vmlinuz64 "$OUT/vmlinuz64"

echo "== unpack base rootfs =="
cd "$WORK/rootfs"
zcat "$WORK/corepure64.gz" | cpio -id --quiet
cd "$WORK"

echo "== resolve curated X11/flwm/wbar/aterm/uzdoom package set (transitive deps) =="
SEED="Xorg-7.7 Xorg-7.7-bin Xorg-7.7-lib vesa-Xorg.conf Xprogs flwm wbar aterm uzdoom"
> queue.txt
> resolved.txt
for p in $SEED; do echo "$p" >> queue.txt; done

ROUND=0
while [ -s queue.txt ]; do
  ROUND=$((ROUND+1))
  sort -u queue.txt > queue_u.txt && mv queue_u.txt queue.txt
  > next_queue.txt
  while read -r pkg; do
    [ -z "$pkg" ] && continue
    grep -qx "$pkg" resolved.txt 2>/dev/null && continue
    echo "$pkg" >> resolved.txt
    if [ ! -f "tcz/$pkg.tcz" ]; then
      curl -sSf -o "tcz/$pkg.tcz" "$TCZ_URL/$pkg.tcz" || echo "WARN: failed to download $pkg.tcz"
    fi
    dep=$(curl -sSf "$TCZ_URL/$pkg.tcz.dep" 2>/dev/null || true)
    for d in $dep; do
      dname="${d%.tcz}"
      grep -qx "$dname" resolved.txt 2>/dev/null || echo "$dname" >> next_queue.txt
    done
  done < queue.txt
  sort -u next_queue.txt > queue.txt
  [ "$ROUND" -gt 15 ] && { echo "too many rounds, stopping"; break; }
done
echo "== resolved $(wc -l < resolved.txt) packages =="

echo "== extract + merge packages into rootfs =="
for f in tcz/*.tcz; do
  [ -e "$f" ] || continue
  name=$(basename "$f" .tcz)
  dest="extract/$name"
  mkdir -p "$dest"
  unsquashfs -f -d "$dest" "$f" > /dev/null 2>&1 || echo "WARN: failed to extract $name"
  cp -a "$dest/." "$WORK/rootfs/" 2>/dev/null || true
done
du -sh "$WORK/rootfs"

echo "== install NovaOS boot config =="
mkdir -p "$WORK/rootfs/opt"
cp /build-assets/xorg.conf "$WORK/rootfs/opt/xorg.conf"
cp /build-assets/wbar.conf "$WORK/rootfs/opt/wbar.conf"
cp /build-assets/bootlocal.sh "$WORK/rootfs/opt/bootlocal.sh"
chmod +x "$WORK/rootfs/opt/bootlocal.sh"

echo "== repack initramfs =="
cd "$WORK/rootfs"
find . | cpio -o -H newc 2>/dev/null | gzip -1 > "$OUT/novaos-initrd.gz"
ls -la "$OUT/"

echo "== cleanup build artifacts to shrink image layer =="
rm -rf "$WORK"
