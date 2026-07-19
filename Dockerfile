FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    novnc \
    websockify \
    squashfs-tools \
    curl \
    cpio \
    gzip \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY deploy/wbar.conf /build-assets/
COPY deploy/build-tinycore.sh /build-tinycore.sh
RUN chmod +x /build-tinycore.sh && /build-tinycore.sh

# chroot-start.sh copied separately, after the expensive package-resolution
# step above, so editing it doesn't invalidate that step's build cache.
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
