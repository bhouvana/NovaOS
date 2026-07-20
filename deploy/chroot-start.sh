#!/bin/sh
# Runs inside the chroot'd Tiny Core rootfs - native speed, no VM/kernel emulation.
export HOME=/root
export SHELL=/bin/sh
export PATH=/usr/local/bin:/usr/local/sbin:/usr/local/games:/bin:/sbin:/usr/bin:/usr/sbin
export LD_LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib

# Docker defaults RLIMIT_NOFILE to 1048576. aterm's daemonizing fork closes
# every fd up to that limit one syscall at a time before exec'ing the login
# shell - confirmed via strace this takes 7+ seconds and it gets SIGHUP'd
# before finishing, so the shell never starts. A traditional Unix-sized
# limit makes that loop finish in milliseconds instead.
ulimit -n 1024

mkdir -p /root /tmp/.X11-unix
chmod 1777 /tmp/.X11-unix
cp /opt/wbar.conf /root/.wbar 2>/dev/null

# Unlike a typical Linux boot, /tmp here isn't on tmpfs - it's part of the
# container's persistent writable layer, so it survives a `docker restart`.
# A stale /tmp/.X0-lock or /tmp/.X11-unix/X0 left over from the previous run
# makes the fresh Xvfb below silently fail to bind (refuses to start if a
# lock for :0 already exists) while leaving the OLD socket file sitting
# there - which makes the "wait for X socket" check further down a false
# positive (the file exists, but nothing is actually listening on it), so
# every client that tries to connect gets "Can't open display" instead of a
# working desktop. Confirmed by reproducing it directly: killing Xvfb without
# clearing these files reliably breaks the next start. Clearing them
# unconditionally here is safe - there's never a legitimate live Xvfb still
# using them at this point in the boot sequence.
rm -f /tmp/.X0-lock
rm -rf /tmp/.X11-unix
mkdir -p /tmp/.X11-unix
chmod 1777 /tmp/.X11-unix

# Persistence: the one-command install mounts /root and
# /etc/sysconfig/tcedir as real Docker volumes, so your files and Software
# Center installs survive container restarts/upgrades instead of resetting
# every time (the rest of the rootfs stays image-only, so pushing a new
# NovaOS image still gets you the update instead of a stale frozen copy).
# tce-load already records every install-center install to tcedir/onboot.lst
# (that's what real Tiny Core itself calls this file) - replaying it here,
# from the locally-cached .tcz files (no download needed), is exactly how
# Tiny Core's own persistence model works, just triggered by us instead of
# its normal boot sequence, which this image doesn't run.
if [ -s /etc/sysconfig/tcedir/onboot.lst ]; then
  echo "=== NovaOS: restoring Software Center installs from persistent storage ==="
  while read -r ext; do
    [ -z "$ext" ] && continue
    tce-load -i "$ext" 2>&1 | sed 's/^/  /'
  done < /etc/sysconfig/tcedir/onboot.lst
fi

# UZDoom (and GZDoom-family engines generally) implement mouselook by
# repeatedly warping the cursor back to screen center and reading the delta
# each frame - a technique that assumes the input device is a real, local
# mouse. Over VNC, the client (browser) reports the ABSOLUTE position of the
# user's actual cursor, with no awareness of the server-side warp-to-center;
# every frame looks like a huge jump from center, so the view snaps wildly.
# This is a fundamental mismatch between warp-based mouselook and any
# absolute-position remote input protocol, not something tunable away - the
# reliable fix is disabling mouse look by default (classic arrow-key
# controls still work exactly as in vanilla Doom). Written before first
# launch so it's the default for a fresh container; doesn't overwrite if the
# player has already customized it.
mkdir -p /root/.config/uzdoom
if [ ! -f /root/.config/uzdoom/uzdoom.ini ]; then
  cat > /root/.config/uzdoom/uzdoom.ini << 'EOF'
[GlobalSettings]
use_mouse=false
m_use_mouse=0
in_mouse=0
m_pitch=0.000000
m_yaw=0.000000
mouse_sensitivity=0.000000
EOF
fi

# Wrapper for launching a terminal, used everywhere instead of calling
# aterm/urxvt directly (wbar.conf, the flwm menu below). Bakes in both aterm
# fixes: strace -f works around a real fork/session-setup race that kills it
# silently within ~1s untraced (see the longer comment further down), and
# -e /bin/sh sidesteps aterm's hardcoded /bin/bash default, which doesn't
# exist in this rootfs. With no arguments it opens an interactive shell;
# with arguments it runs that command in the terminal (e.g. `nova-term htop`).
cat > /usr/local/bin/nova-term << 'EOF'
#!/bin/sh
if [ $# -eq 0 ]; then
  exec strace -f -o /dev/null aterm -geometry 90x28+40+40 -e /bin/sh
else
  exec strace -f -o /dev/null aterm -geometry 90x28+40+40 -e "$@"
fi
EOF
chmod +x /usr/local/bin/nova-term
cat > /usr/local/bin/nova-term-rxvt << 'EOF'
#!/bin/sh
if [ $# -eq 0 ]; then
  exec strace -f -o /dev/null urxvt -e /bin/sh
else
  exec strace -f -o /dev/null urxvt -e "$@"
fi
EOF
chmod +x /usr/local/bin/nova-term-rxvt

# Software Center: a real, working "install more" path, not just what's
# baked into the image. Backed by TC's own tce-load (patched at build time to
# run as root - see build-tinycore.sh) against the full ~3500-package Tiny
# Core repo. Simple form for the common case, plus a Browse button that opens
# tce-ab (TC's built-in text browser: search/info/install) for anyone who
# wants to explore rather than type an exact name.
cat > /usr/local/bin/nova-software-center << 'EOF'
#!/bin/sh
export DISPLAY=:0
export PATH=/usr/local/bin:/usr/local/sbin:/usr/local/games:/bin:/sbin:/usr/bin:/usr/sbin
OUT=$(yad --form --title="NovaOS Software Center" --width=460 --center \
  --image="applications-other" \
  --text="Install more software from Tiny Core's repository (3500+ packages).\nEnter an exact package name (e.g. blender, krita, dosbox), or Browse to search." \
  --field="Package name" "" \
  --button="Browse All!!!Opens a text-based search/browse tool in a terminal:2" \
  --button="Cancel:1" --button="Install:0")
RET=$?
case $RET in
  0) NAME=$(echo "$OUT" | cut -d'|' -f1)
     [ -n "$NAME" ] && nova-term sh -c "tce-load -wi '$NAME'; echo; echo Press enter to close.; read x"
     ;;
  2) nova-term tce-ab ;;
esac
EOF
chmod +x /usr/local/bin/nova-software-center

# Universal app launcher: Alt+Space (bound below via sxhkd) pops up a
# fuzzy-search list of every app in the right-click menu and runs whatever's
# selected. Reads the $HOME/.wmx tree built by additem() below instead of
# keeping a second hardcoded list in sync with it - one source of truth for
# both the right-click menu and the launcher, so a new additem() call is
# automatically searchable too. Originally used rofi (not GTK-based, so it
# sidesteps the icon-cache crash class of bug that hit geany/pcmanfm), but
# rofi's cairo/xcb-render pipeline renders a solid black window with no text
# at all under this Xvfb setup (confirmed: window maps, background paints,
# but nothing else draws - reproduced even with a plain 3-item test list and
# several theme/visual overrides, so it's a rendering bug in that specific
# build, not a config issue). Switched to dmenu - plain Xlib+Xft, no
# compositing pipeline to break, confirmed working. Explicit LD_LIBRARY_PATH
# here because sxhkd-spawned children aren't guaranteed to inherit
# chroot-start.sh's exported env (confirmed: without it, dmenu/rofi fail
# silently with no on-screen indication at all).
cat > /usr/local/bin/nova-launcher << 'EOF'
#!/bin/sh
export DISPLAY=:0
export PATH=/usr/local/bin:/usr/local/sbin:/usr/local/games:/bin:/sbin:/usr/bin:/usr/sbin
export LD_LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib
LIST=$(for f in "$HOME"/.wmx/*/*; do
  [ -f "$f" ] || continue
  label=$(basename "$f")
  cmd=$(sed -n 's/^exec //p' "$f")
  printf '%s\t%s\n' "$label" "$cmd"
done)
CHOICE=$(printf '%s\n' "$LIST" | cut -f1 | dmenu -i -p "NovaOS")
[ -z "$CHOICE" ] && exit 0
CMD=$(printf '%s\n' "$LIST" | awk -F'\t' -v c="$CHOICE" '$1 == c { print $2; exit }')
[ -n "$CMD" ] && sh -c "exec $CMD" &
EOF
chmod +x /usr/local/bin/nova-launcher

# sxhkd binds the actual hotkey. Alt+Space rather than the "Super"/Windows
# key most real launchers use - Super is frequently intercepted by the host
# OS or browser before a remote session ever sees it (Windows' own Start
# Menu, for one), so it's not reliable inside a browser tab. Alt+Space isn't
# claimed by anything at that layer.
mkdir -p /root/.config/sxhkd
cat > /root/.config/sxhkd/sxhkdrc << 'EOF'
alt + space
    nova-launcher
EOF

echo "=== NovaOS: starting Xvfb (virtual framebuffer, no VM needed) ==="
# -listen tcp: lets processes outside the chroot (which have no visibility into
# this chroot's /tmp/.X11-unix socket) reach this X server over 127.0.0.1:6000.
# Full 1080p at 24-bit color - no longer holding back for a RAM/CPU-constrained
# cloud target, this now runs locally with real resources. RANDR + x11vnc's
# -xrandr (below) let noVNC's resize=remote grow/shrink the desktop to match
# whatever size the browser window actually is.
Xvfb :0 -screen 0 1920x1080x24 +extension GLX +extension RANDR -listen tcp &

echo "=== NovaOS: waiting for X socket ==="
for i in 1 2 3 4 5 6 7 8 9 10; do
  [ -S /tmp/.X11-unix/X0 ] && { echo "X socket ready after ${i}s"; break; }
  sleep 1
done

export DISPLAY=:0

# --- build the flwm right-click desktop menu -------------------------------
# flwm reads its root-window menu from a directory tree at $HOME/.wmx: each
# subdirectory becomes a submenu, each executable file inside it becomes a
# clickable item that runs "exec <command>" when clicked (this is the same
# mechanism TC's own flwm_makemenu/flwm_initmenu scripts use, just driven
# directly here instead of via .desktop-file parsing since we want full
# control over a curated menu covering the whole package set).
WMX="$HOME/.wmx"
rm -rf "$WMX"
additem() {
  # additem "Category" "Label" "command"
  d="$WMX/$1"
  mkdir -p "$d"
  f="$d/$2"
  printf '#!/bin/sh\nexec %s\n' "$3" > "$f"
  chmod +x "$f"
}

additem "Terminal" "aterm"           "nova-term"
additem "Terminal" "rxvt"            "nova-term-rxvt"

# A few packages in the curated set are installed but not wired into any
# menu/taskbar entry, because launching them does nothing useful:
#  - xarchiver: missing libglapi.so.0 at runtime - no package in TC's current
#    x86_64 repo actually ships that file (checked all the obvious Mesa/GL
#    candidates), so this looks like a stale .tcz built against an older
#    Mesa layout. engrampa covers the same job and its own deps resolve
#    clean, so it's the one on the menu.
#  - filezilla: linked against wxWidgets 3.0 (libwx_gtk3u_*-3.0.so.0), but
#    the wxwidgets.tcz this repo currently serves is 3.2 - no 3.0 build is
#    hosted anywhere in the repo to satisfy it. lftp (terminal) covers FTP
#    instead.
#  - vlc: refuses to run as root outright ("VLC is not supposed to be run
#    as root") and its own vlc-wrapper bypass needs an unprivileged user
#    account to drop to, which this single-root-user container doesn't
#    have. mpv has no such restriction and is the primary player here; vlc
#    stays installed (harmless) but off the menu.
additem "Files" "File Manager"       "pcmanfm"
additem "Files" "Archive Manager"    "engrampa"

additem "Editors" "Leafpad"          "leafpad"
additem "Editors" "Geany"            "geany"
additem "Editors" "Bluefish (Web)"   "bluefish"
additem "Editors" "Diff/Merge (meld)" "meld"
additem "Editors" "Lite XL"          "lite-xl"
additem "Editors" "Vim (terminal)"   "nova-term vim"
additem "Editors" "Nano (terminal)"  "nova-term nano"

additem "Graphics" "GIMP"            "gimp-3.0"
additem "Graphics" "Inkscape"        "inkscape"
additem "Graphics" "mtPaint"         "mtpaint"
additem "Graphics" "Image Viewer"    "gpicview"
additem "Graphics" "Color Picker"    "gcolor2"
additem "Graphics" "Darktable"       "darktable"
additem "Graphics" "Photo Manager"   "shotwell"
additem "Graphics" "Image Browser"   "xzgv"
additem "Graphics" "gThumb"          "gthumb"
additem "Graphics" "Photoflare"      "photoflare"
additem "Graphics" "Screen Recorder" "simplescreenrecorder"

additem "Office" "LibreOffice Writer"   "lowriter"
additem "Office" "LibreOffice Calc"     "localc"
additem "Office" "LibreOffice Impress"  "loimpress"
additem "Office" "AbiWord"              "abiword"
additem "Office" "Gnumeric"             "gnumeric"
additem "Office" "Document Viewer"      "evince"
additem "Office" "Calculator"           "galculator"
additem "Office" "PDF Viewer (mupdf)"   "mupdf"

additem "Internet" "Firefox"         "firefox"
additem "Internet" "Thunderbird Mail" "thunderbird"
additem "Internet" "HexChat IRC"     "hexchat"
additem "Internet" "FTP (lftp, terminal)" "nova-term lftp"
additem "Internet" "qBittorrent"     "qbittorrent"
additem "Internet" "PuTTY (SSH)"     "putty"
additem "Internet" "Remmina (Remote Desktop)" "remmina"
additem "Internet" "IRC (irssi, terminal)" "nova-term irssi"
additem "Internet" "IRC (weechat, terminal)" "nova-term weechat"
additem "Internet" "Transmission (Torrent)" "transmission-gtk"
additem "Internet" "Sylpheed Mail"   "sylpheed"
additem "Internet" "uGet Download Manager" "uget-gtk"
additem "Internet" "Profanity (XMPP, terminal)" "nova-term profanity"

additem "Multimedia" "mpv"           "mpv"
additem "Multimedia" "Audacious"     "audacious"
additem "Multimedia" "Audacity"      "audacity"
additem "Multimedia" "HandBrake"     "ghb"
additem "Multimedia" "mpg123 (terminal)" "nova-term mpg123"
additem "Multimedia" "DeaDBeeF"      "deadbeef"
additem "Multimedia" "QMPlay2"       "qmplay2"
additem "Multimedia" "Brasero (Disc Burning)" "brasero"
additem "Multimedia" "Asunder (CD Ripper)" "asunder"

additem "System Tools" "System Monitor (htop)" "nova-term htop"
additem "System Tools" "Partition Editor"      "gparted"
additem "System Tools" "Wireshark"             "wireshark-gtk"
additem "System Tools" "System Info (neofetch)" "nova-term neofetch"
additem "System Tools" "System Info (fastfetch)" "nova-term fastfetch"
additem "System Tools" "System Info (inxi)"    "nova-term inxi -Fz"
additem "System Tools" "Disk Usage (ncdu)"     "nova-term ncdu /"
additem "System Tools" "Data Recovery (testdisk)" "nova-term testdisk"
additem "System Tools" "Developer Terminal"    "nova-term"
additem "System Tools" "Software Center"       "nova-software-center"

additem "Games" "Doom"               "uzdoom"
additem "Games" "Luanti (Minetest)"  "minetest"
additem "Games" "SuperTux"           "supertux2"
additem "Games" "Neverball"          "neverball"
additem "Games" "Neverputt (mini golf)" "neverputt"
additem "Games" "Bubble Shooter"     "xbubble"
additem "Games" "Minesweeper"        "gnome-mines"
additem "Games" "Chess"              "cutechess"
additem "Games" "Sudoku (terminal)"  "nova-term sudoku"
additem "Games" "Solitaire"          "solitaire"
additem "Games" "FreeCell"           "freecell"
additem "Games" "Spider Solitaire"   "spider"
additem "Games" "Mahjong (Taipei)"   "taipei"
additem "Games" "Mastermind"         "mastermind"
additem "Games" "Mini Golf (Ace)"    "golf"
additem "Games" "DOSBox-X"           "dosbox-x -nopromptfolder"
additem "Games" "MAME (Arcade)"      "mame64"
additem "Games" "SNES9x"             "snes9x-gtk"
additem "Games" "PipeWalker"         "pipewalker"
additem "Games" "LBreakoutHD"        "lbreakouthd"

additem "Programming" "Ruby (terminal)"   "nova-term ruby"
additem "Programming" "Node.js (terminal)" "nova-term node"
additem "Programming" "Python (terminal)" "nova-term python3.9"
additem "Programming" "PHP (terminal)"    "nova-term php"
additem "Programming" "Lua (terminal)"    "nova-term lua"
additem "Programming" "Go (terminal)"     "nova-term sh -c 'go version; exec sh'"
additem "Programming" "R (terminal)"      "nova-term R"
additem "Programming" "GDB (terminal)"    "nova-term gdb"

# Desktop icons: mirrors the $WMX tree just built above into
# ~/Desktop/<Category>/<Label>.desktop launcher files, so pcmanfm's desktop
# manager (below) shows the exact same categorized app list as the
# right-click menu and Alt+Space launcher, as double-clickable folder icons -
# one more view onto the same additem() calls rather than a fourth list to
# keep in sync. Icons are looked up from wbar.conf's existing i:/c: pairs by
# matching the command (covers the original ~50 taskbar apps, since their
# additem() and wbar.conf commands are kept identical on purpose); anything
# without a match falls back to a generic icon rather than a broken one.
echo "=== NovaOS: generating desktop icons ==="
rm -rf "$HOME/Desktop"
mkdir -p "$HOME/Desktop"
GENERIC_ICON=/usr/local/share/icons/hicolor/48x48/apps/utilities-terminal.png
[ -f "$GENERIC_ICON" ] || GENERIC_ICON=/usr/local/share/pixmaps/geany.png
find "$HOME/.wmx" -type f | while read -r f; do
  category=$(basename "$(dirname "$f")")
  label=$(basename "$f")
  cmd=$(sed -n 's/^exec //p' "$f")
  icon=$(awk -v cmd="$cmd" '
    /^i:/ { icon = substr($0, 4) }
    /^c:/ { if (substr($0, 4) == cmd) { print icon; exit } }
  ' /opt/wbar.conf)
  [ -z "$icon" ] && icon="$GENERIC_ICON"
  destdir="$HOME/Desktop/$category"
  mkdir -p "$destdir"
  cat > "$destdir/$label.desktop" << EOF2
[Desktop Entry]
Type=Application
Name=$label
Exec=$f
Icon=$icon
Terminal=false
EOF2
  chmod +x "$destdir/$label.desktop"
done

# pcmanfm's own -w/--wallpaper-mode CLI flags don't reliably take effect on
# a first launch (confirmed: it silently falls back to wallpaper_mode=color
# with no wallpaper= key written at all) - writing its config file directly
# before launch is what actually works.
mkdir -p "$HOME/.config/pcmanfm/default"
if [ -f /usr/local/share/novaos-wallpaper.png ]; then
  WALLPAPER_LINE="wallpaper=/usr/local/share/novaos-wallpaper.png"
  WALLPAPER_MODE="crop"
else
  WALLPAPER_LINE=""
  WALLPAPER_MODE="color"
fi
cat > "$HOME/.config/pcmanfm/default/desktop-items-0.conf" << EOF
[*]
wallpaper_mode=$WALLPAPER_MODE
wallpaper_common=1
$WALLPAPER_LINE
desktop_bg=#4a6fa5
desktop_fg=#ffffff
desktop_shadow=#000000
desktop_font=Sans 12
show_wm_menu=0
sort=mtime;ascending;
show_documents=0
show_trash=1
show_mounts=0
EOF

echo "=== NovaOS: launching flwm ==="
flwm &
sleep 1
echo "=== NovaOS: launching desktop icons + wallpaper (pcmanfm) ==="
pcmanfm --desktop &
# wbar's dock skin (osxbarback.png) uses pseudo-transparency - it grabs the
# root window's background pixmap (via the standard _XROOTPMAP_ID property)
# and blends that into its own rendering instead of drawing a real
# translucent surface. pcmanfm's own wallpaper mechanism doesn't set that
# property (it paints its own window directly), so wbar had nothing valid
# to grab and fell back to solid black behind every icon - confirmed
# empirically: still had the black box with pcmanfm's own wallpaper
# genuinely working. feh does set that property, same as it always did, so
# running it alongside pcmanfm gives wbar something real to blend with again
# without changing what's actually visible for the rest of the desktop
# (pcmanfm's own icon-desktop window still paints on top of it either way).
if [ -f /usr/local/share/novaos-wallpaper.png ]; then
  feh --bg-scale /usr/local/share/novaos-wallpaper.png
fi
echo "=== NovaOS: launching wbar ==="
wbar -pos bottom -isize 32 &

# Desktop overlay: clock/date and basic system stats in the corner, in the
# spirit of a real desktop OS rather than a bare taskbar on a flat color.
cat > /root/.conkyrc << 'EOF'
conky.config = {
    update_interval = 1,
    double_buffer = true,
    own_window = true,
    own_window_type = 'desktop',
    own_window_transparent = true,
    own_window_hints = 'undecorated,below,sticky,skip_taskbar,skip_pager',
    border_width = 0,
    alignment = 'top_right',
    gap_x = 24,
    gap_y = 24,
    minimum_width = 260,
    default_color = 'white',
    draw_shades = true,
    default_shade_color = 'black',
    use_xft = true,
    font = 'DejaVu Sans:size=10',
};
conky.text = [[
${font DejaVu Sans:bold:size=18}${time %H:%M:%S}${font}
${font DejaVu Sans:size=11}${time %A, %B %d %Y}${font}
${hr 1}
${font DejaVu Sans:bold:size=10}NovaOS${font} - Tiny Core, fully loaded
Uptime: $uptime
CPU: $cpu%  Mem: $mem / $memmax
]];
EOF
conky &

# flwm doesn't implement the _NET_WM_WINDOW_TYPE_DESKTOP EWMH hint (it's a
# minimal, pre-EWMH-era window manager), so pcmanfm's desktop-icon window -
# which relies on that hint to know it belongs at the very bottom of the
# stack - ends up on top of everything instead, hiding wbar and conky behind
# solid black (confirmed empirically). A one-shot raise right after launch
# isn't enough either - confirmed empirically that wbar/conky can end up
# covered again sometime after boot even once successfully raised once
# (pcmanfm re-asserting its own stacking position, most likely). Runs as a
# standing background loop for the life of the session instead of a single
# fix-up, so it self-heals whenever this happens rather than only covering
# the boot race. Cheap - two xdotool searches every few seconds.
(
  while true; do
    WBAR_WIN=$(xdotool search --name wbar 2>/dev/null | head -1)
    [ -n "$WBAR_WIN" ] && xdotool windowraise "$WBAR_WIN" 2>/dev/null
    CONKY_WIN=$(xdotool search --class conky 2>/dev/null | head -1)
    [ -n "$CONKY_WIN" ] && xdotool windowraise "$CONKY_WIN" 2>/dev/null
    sleep 3
  done
) &

echo "=== NovaOS: launching sxhkd (Alt+Space app launcher) ==="
sxhkd &

# --- terminal: only works with a real devpts mount ---------------------
# aterm/urxvt need a Unix98 PTY (ptsname() -> open(/dev/pts/N)), which needs
# a devpts instance mounted in THIS mount namespace. Unprivileged containers
# can't mount one (confirmed on Render), but this image is now meant to run
# locally with elevated privileges (see README / the one-line docker run
# command, which passes --privileged), so check for it and use it if present
# instead of always skipping.
#
# Separately, two real bugs needed fixing to get a working shell here:
#  1. aterm's daemonizing fork closes every fd up to RLIMIT_NOFILE one
#     syscall at a time before exec'ing a shell - Docker defaults that to
#     1048576, so the close loop took 7+ seconds and aterm got SIGHUP'd
#     before ever reaching the exec. Fixed by the `ulimit -n 1024` above.
#  2. aterm's default shell guess is a hardcoded /bin/bash, which doesn't
#     exist in this minimal Tiny Core rootfs (only busybox's /bin/sh) -
#     confirmed via strace: execve("/bin/bash", ...) = -1 ENOENT, then
#     aterm prints "can't execute" to the pty and exits. Fixed by passing
#     -e /bin/sh explicitly instead of relying on the default.
#  Also: aterm has a separate, genuine timing race in its own fork/session
#  setup that kills it silently within ~1s when run at full native speed
#  (verified via strace - it reaches a stable event loop and runs
#  indefinitely when traced, but dies untraced with no error output).
#  Running it under a no-op strace wrapper reliably avoids that race - a
#  known, if inelegant, workaround for this class of bug in old X11
#  binaries. Tracing overhead is negligible for a terminal's syscall volume.
echo "=== NovaOS: desktop launch sequence complete, starting x11vnc ==="
if mountpoint -q /dev/pts 2>/dev/null; then
  echo "=== NovaOS: devpts is mounted - launching terminal ==="
  strace -f -o /dev/null aterm -geometry 90x28+40+40 -e /bin/sh &
else
  echo "=== NovaOS: no devpts mount - terminal unavailable this run (run with --privileged or --cap-add=SYS_ADMIN to enable it) ==="
fi

# -xrandr resize: lets a client asking for resize=remote (server-side RandR
# resize) get it. The default entry page (index.html) uses resize=scale
# instead - pure client-side canvas scaling, so the desktop always fits the
# browser window exactly with no scrollbars regardless of screen size,
# without depending on RandR renegotiation timing. Kept here too since it's
# harmless and some clients may still prefer server-side resize.
#
# -defer/-wait: x11vnc's own defaults are 20ms/20ms; this was previously set
# to 40/40 (double the default) specifically to cut CPU/network load for
# Render's constrained free-tier CPU - a real tradeoff at the time, but this
# now runs locally with a real machine's resources, so that tradeoff was
# just adding latency to every screen update for no benefit anymore. Cut
# well below even x11vnc's own default given how much headroom a local
# machine has compared to what these were originally tuned for. -threads
# handles input/output on separate threads per client instead of one,
# which matters more once updates are this much more frequent.
exec x11vnc -display :0 -forever -shared -rfbport 5900 -nopw -quiet \
  -defer 5 -wait 5 -threads -nocursorshape -nobell -xrandr resize
