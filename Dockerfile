FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    qemu-system-x86 \
    novnc \
    websockify \
    squashfs-tools \
    curl \
    cpio \
    gzip \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY deploy/xorg.conf deploy/wbar.conf deploy/bootlocal.sh /build-assets/
COPY deploy/build-tinycore.sh /build-tinycore.sh
RUN chmod +x /build-tinycore.sh /build-assets/bootlocal.sh && /build-tinycore.sh

COPY deploy/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 8080
CMD ["/entrypoint.sh"]
