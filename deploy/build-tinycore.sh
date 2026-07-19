#!/bin/bash
# Runs during `docker build`. Fetches Tiny Core's curated desktop package set
# and merges it into a real rootfs directory (no kernel, no initramfs) - the
# container runs this rootfs directly via chroot at runtime, native speed,
# no nested VM/CPU emulation at all.
set -euo pipefail

WORK=/build
BASE_URL=http://tinycorelinux.net/16.x/x86_64
TCZ_URL="$BASE_URL/tcz"
OUT=/opt/novaos/tc-root

mkdir -p "$WORK/rootfs" "$WORK/tcz" "$WORK/extract"
mkdir -p "$OUT"
cd "$WORK"

echo "== fetching Tiny Core base rootfs (userland only, no kernel needed) =="
curl -sSf -o corepure64.gz "$BASE_URL/release/distribution_files/corepure64.gz"

echo "== unpack base rootfs =="
cd "$WORK/rootfs"
zcat "$WORK/corepure64.gz" | cpio -id --quiet
cd "$WORK"

echo "== resolve curated full-desktop package set (transitive deps, parallel) =="
SEED="Xorg-7.7 Xorg-7.7-bin Xorg-7.7-lib Xorg-7.7-3d Xprogs flwm wbar aterm uzdoom pcmanfm leafpad geany gpicview galculator abiword midori mtpaint x11vnc"
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
  echo "  round $ROUND: resolving $(wc -l < queue.txt) new packages in parallel"

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
      dname="${d%.tcz}"
      echo "$dname" >> next_queue.txt
    done
  done < queue.txt
  sort -u next_queue.txt | comm -23 - <(sort -u resolved.txt) > queue.txt
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
cp /build-assets/wbar.conf "$WORK/rootfs/opt/wbar.conf"

echo "== move rootfs into place as a real directory tree (no packing) =="
mkdir -p /tmp/dev /tmp/proc /tmp/sys  # placeholders; real ones bind-mounted at runtime
rm -rf "$OUT"
mv "$WORK/rootfs" "$OUT"
mkdir -p "$OUT/dev" "$OUT/proc" "$OUT/sys" "$OUT/tmp"
du -sh "$OUT"

echo "== cleanup build artifacts to shrink image layer =="
rm -rf "$WORK"
