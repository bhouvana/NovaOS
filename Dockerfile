FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    novnc \
    websockify \
    squashfs-tools \
    curl \
    cpio \
    gzip \
    ca-certificates \
    imagemagick \
    && rm -rf /var/lib/apt/lists/*

COPY deploy/build-tinycore.sh /build-tinycore.sh
RUN chmod +x /build-tinycore.sh && /build-tinycore.sh

# Minetest has no pre-built Tiny Core package, so it's compiled from source
# in its own layer - separate from the step above so editing this script
# doesn't force the (already expensive) main package resolution to redo.
COPY deploy/build-minetest.sh /build-minetest.sh
RUN chmod +x /build-minetest.sh && /build-minetest.sh

# wbar.conf and chroot-start.sh copied separately, after both expensive steps
# above, so editing either one (which happens often - taskbar/menu tweaks)
# doesn't force a full package re-resolution or, worse, a full Minetest
# recompile. Learned this the hard way: wbar.conf used to be consumed by
# build-tinycore.sh directly, so editing it invalidated everything downstream
# including the ~15+ minute Minetest build.
COPY deploy/wbar.conf /opt/novaos/tc-root/opt/wbar.conf
COPY deploy/chroot-start.sh /opt/novaos/tc-root/opt/chroot-start.sh
RUN chmod +x /opt/novaos/tc-root/opt/chroot-start.sh

COPY deploy/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# noVNC's own entry point is vnc.html, not index.html - without this, visiting
# the bare domain root just shows websockify's directory listing instead of
# the desktop. Copied last so editing it doesn't invalidate earlier cached
# layers (especially the expensive package-resolution step).
COPY deploy/index.html /usr/share/novnc/index.html

EXPOSE 8080
CMD ["/entrypoint.sh"]
