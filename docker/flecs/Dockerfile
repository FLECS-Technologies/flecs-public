FROM debian:bookworm-slim AS docker-downloader

RUN apt-get update && \
    apt-get --yes --no-install-recommends install \
    ca-certificates wget && \
    apt-get --yes clean && \
    rm -rf /var/lib/apt/lists/*

ARG ARCH

RUN mkdir -p /fs/usr/bin
RUN case "${ARCH}" in \
    "amd64") \
    wget -O /fs/usr/bin/docker-compose https://github.com/docker/compose/releases/download/v2.34.0/docker-compose-linux-x86_64; \
    ;; \
    "armhf") \
    wget -O /fs/usr/bin/docker-compose https://github.com/docker/compose/releases/download/v2.34.0/docker-compose-linux-armv7; \
    ;; \
    "arm64") \
    wget -O /fs/usr/bin/docker-compose https://github.com/docker/compose/releases/download/v2.34.0/docker-compose-linux-aarch64; \
    ;; \
    *) \
    exit 1 \
    ;; \
    esac

RUN chmod a+x /fs/usr/bin/docker-compose

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get --yes --no-install-recommends install \
    ca-certificates docker.io git iptables procps nginx && \
    apt-get --yes clean && \
    rm -rf /var/lib/apt/lists/*

RUN update-alternatives --set iptables /usr/sbin/iptables-legacy && \
    update-alternatives --set ip6tables /usr/sbin/ip6tables-legacy

COPY --from=docker-downloader /fs /
ADD fs/ /

RUN echo "/usr/local/lib" >/etc/ld.so.conf.d/00-flecs.conf && ldconfig

ENTRYPOINT [ "/entrypoint.sh" ]
