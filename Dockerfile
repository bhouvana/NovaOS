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

# Minimal device nodes for the chroot'd Tiny Core rootfs - created at build time
# so no runtime mount privileges are required (Render doesn't grant CAP_SYS_ADMIN).
# Tiny Core's own base rootfs already ships some of these - skip if present.
RUN mkdir -p /opt/novaos/tc-root/dev/pts && cd /opt/novaos/tc-root/dev \
    && [ -e null ] || mknod -m 666 null c 1 3 \
    && [ -e zero ] || mknod -m 666 zero c 1 5 \
    && [ -e random ] || mknod -m 666 random c 1 8 \
    && [ -e urandom ] || mknod -m 666 urandom c 1 9 \
    && [ -e tty ] || mknod -m 666 tty c 5 0 \
    && [ -e ptmx ] || mknod -m 666 ptmx c 5 2

COPY deploy/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 8080
CMD ["/entrypoint.sh"]
