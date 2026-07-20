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

echo "== unpack base rootfs (skip dev/* - sandboxed builders reject mknod at build time) =="
cd "$WORK/rootfs"
zcat "$WORK/corepure64.gz" | cpio -id --quiet -f 'dev/*'
mkdir -p dev
cd "$WORK"

echo "== resolve curated full-desktop package set (transitive deps, parallel) =="
# "Ultimate edition" - the full desktop: window manager/taskbar, terminal,
# file/text/code tools, graphics, office suite, documents, internet, media,
# system/dev tools, and (Doom plus 6 more) games. Deliberately excludes
# anything that would fight what's already here rather than add to it:
# other window managers/desktop environments (icewm, jwm, fluxbox, openbox,
# i3, gnome-shell/mutter, xfce...) since flwm is the one actually wired up
# to wbar/sxhkd/the right-click menu; other audio/display servers
# (pipewire vs the pulseaudio stack already pulled in transitively); and
# full alternative browser engines (firefox/chromium-class packages) given
# how much sandboxing plumbing they expect that a chroot doesn't provide -
# midori/dillo/netsurf/seamonkey already cover that ground at a size and
# risk this build is comfortable with.
SEED="Xorg-7.7 Xorg-7.7-bin Xorg-7.7-lib Xorg-7.7-3d Xprogs flwm wbar \
  aterm rxvt \
  pcmanfm leafpad geany gimp inkscape mtpaint gpicview gcolor2 \
  galculator abiword libreoffice gnumeric evince xarchiver \
  midori dillo netsurf seamonkey thunderbird hexchat filezilla qbittorrent \
  vlc mpv audacious mpg123 \
  htop gparted wireshark git gcc make python3.9 nmap gnupg openssh rsync screen tmux neofetch figlet strace \
  uzdoom supertux nevergames xbubble ace-of-penguins gnome-mines cutechess sudoku dosbox-x \
  conky yad feh dmenu sxhkd \
  darktable irssi meld putty remmina shotwell weechat xzgv audacity bluefish handbrake \
  x11vnc \
  ruby perl5 php-8.3-cli lua-5.4 tcl8.6 go R node sqlite3-bin valgrind gdb cmake meson ninja \
  ccache clang llvm-bin mono swig boost-1.84 jq tig mercurial svn pgloader doxygen \
  autoconf automake libtool bison flex \
  vim nano joe l3afpad lite-xl zile ed beaver ted \
  gthumb eog photoflare simplescreenrecorder scrot imagemagick graphicsmagick potrace \
  ufraw dcraw exiv2 jpegoptim gimagereader \
  xpdf-tools mupdf qpdf pdftk img2pdf texinfo groff \
  transmission sylpheed profanity links-full aria2 uget rtorrent lftp \
  mplayer-cli deadbeef timidity ffmpeg7 asunder abcde cdrtools brasero qmplay2 g4music \
  freedoom mame snes9x pipewalker lbreakouthd mednafen \
  engrampa \
  lshw hwloc fastfetch inxi ncdu tree lsof iftop mtr iperf3 speedtest-cli smartmontools \
  powertop sysstat tcpdump hashcat testdisk ddrescue exfatprogs ntfs-3g samba \
  dejavu-fonts-ttf liberation-fonts-ttf terminus-fonts nerd-fonts-ttf Hack-font \
  adwaita-icon-theme humanity-icon-theme oxygen-fonts ttf-bitstream-vera \
  7zip zip unrar lz4 zstd unzip"
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
  [ "$ROUND" -gt 25 ] && { echo "too many rounds, stopping"; break; }
done
echo "== resolved $(wc -l < resolved.txt) packages =="

echo "== extract packages (parallel) =="
ls tcz/*.tcz | xargs -P 8 -I{} sh -c '
  f="{}"
  name=$(basename "$f" .tcz)
  dest="extract/$name"
  mkdir -p "$dest"
  unsquashfs -f -d "$dest" "$f" > /dev/null 2>&1 || echo "WARN: failed to extract $name"
'

echo "== merge extracted packages into rootfs =="
for d in extract/*/; do
  cp -a "$d." "$WORK/rootfs/" 2>/dev/null || true
done
du -sh "$WORK/rootfs"

echo "== set up tce-load's expected extension directory =="
# tce-load silently `exit 1`s (no message at all) if /etc/sysconfig/tcedir
# isn't a real directory - normally set up by TC's own boot process
# (tc-config), which this image bypasses entirely by chrooting straight in.
mkdir -p "$WORK/rootfs/etc/sysconfig/tcedir/optional"
touch "$WORK/rootfs/etc/sysconfig/tcedir/onboot.lst"

echo "== allow tce-load/tce-ab to run as root =="
# TC's package-management tools refuse to run as root (checknotroot in
# tc-functions) because they normally expect a non-root "tc" user - this is a
# single-user root-only desktop container, so that model doesn't apply. This
# is what lets the in-desktop Software Center actually install more packages
# live instead of just failing with "Don't run this as root." Appending the
# override to the end of tc-functions makes it win regardless of where the
# file gets sourced from - shell re-definitions take the last one seen.
printf '\nchecknotroot() { return 0; }\n' >> "$WORK/rootfs/etc/init.d/tc-functions"

echo "== run package post-install scripts =="
# Every .tcz under /usr/local/tce.installed/<name> ships a script meant to be
# run once, right after that package is installed (tce-load's normal job) -
# symlinks, icon-theme cache, GSettings schema compilation, font cache,
# mime database, desktop-file database, etc. Building by mass-extracting and
# merging packages ourselves skips all of that, which is what was causing
# hard crashes in GTK3 apps (geany, pcmanfm) on startup: gdk-pixbuf's loader
# cache was never generated, so GTK couldn't decode ANY stock icon and hit a
# fatal assertion. Running every script now, in one chroot, fixes this and
# whatever else the other ~65 scripts handle (fonts, mime types, etc).
# A handful of these scripts assume TC's normal live model, where each .tcz
# stays loop-mounted at /tmp/tcloop/<name>/ and installed files are symlinks
# into that mount - not our merged-real-filesystem approach. Running them
# here is actively destructive: libfm's script does `rm -rf
# /usr/local/lib/libfm*so*` then relinks to that (nonexistent) loop path,
# deleting the real, correctly-extracted libraries and leaving a broken
# symlink; nevergames' does the same to its own game-data directory, wiping
# Neverball/Neverputt's assets. Confirmed via grep -l tcloop across all
# scripts - skip any script that references it, plain extraction already
# left those packages in a correct state.
chroot "$WORK/rootfs" /usr/bin/env LD_LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib \
  PATH=/usr/local/bin:/usr/local/sbin:/bin:/sbin:/usr/bin:/usr/sbin \
  /bin/sh -c 'for f in /usr/local/tce.installed/*; do
    [ -f "$f" ] || continue
    if grep -q "tcloop" "$f" 2>/dev/null; then
      echo "  skipping (assumes live tcloop mount): $(basename "$f")"
      continue
    fi
    echo "  running: $(basename "$f")"
    sh "$f" 2>&1 | sed "s/^/    /"
  done' || echo "WARN: some post-install scripts failed, continuing anyway"

echo "== fix up packages whose postinstall script we skipped but that had a real, non-destructive side effect =="
# audacious's postinstall script is skipped above because it references
# /tmp/tcloop (like libfm/nevergames), but unlike those it's not destructive -
# it just copies the real binary from the extracted package into
# /usr/local/bin (apparently upstream's own build has "some strange activity"
# that leaves it out otherwise). Skipping the whole script means that copy
# never happens and the binary is left stranded under
# /usr/local/share/audacious/files/, invisible to $PATH and any menu entry.
# Redo just that part here, directly against the merged rootfs.
AUD_BIN="$WORK/rootfs/usr/local/share/audacious/files/audacious"
if [ -x "$AUD_BIN" ] && [ ! -e "$WORK/rootfs/usr/local/bin/audacious" ]; then
  cp "$AUD_BIN" "$WORK/rootfs/usr/local/bin/audacious"
  chmod 755 "$WORK/rootfs/usr/local/bin/audacious"
fi

echo "== generate NovaOS wallpaper =="
convert -size 1920x1080 gradient:'#1a2744-#4a6fa5' \
  -gravity center -fill 'rgba(255,255,255,0.15)' -pointsize 220 -font DejaVu-Sans-Bold \
  -annotate 0 "NovaOS" \
  -gravity south -fill 'rgba(255,255,255,0.35)' -pointsize 24 -font DejaVu-Sans \
  -annotate +0+60 "Tiny Core, fully loaded" \
  "$WORK/rootfs/usr/local/share/novaos-wallpaper.png"

mkdir -p "$WORK/rootfs/opt"

echo "== move rootfs into place as a real directory tree (no packing) =="
mkdir -p /tmp/dev /tmp/proc /tmp/sys  # placeholders; real ones bind-mounted at runtime
rm -rf "$OUT"
mv "$WORK/rootfs" "$OUT"
mkdir -p "$OUT/dev" "$OUT/proc" "$OUT/sys" "$OUT/tmp"
du -sh "$OUT"

echo "== cleanup build artifacts to shrink image layer =="
rm -rf "$WORK"
