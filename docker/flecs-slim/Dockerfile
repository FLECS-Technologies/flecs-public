FROM debian:bookworm-slim AS docker-downloader

RUN apt-get update && \
    apt-get --yes --no-install-recommends install \
    ca-certificates wget && \
    apt-get --yes clean && \
    rm -rf /var/lib/apt/lists/*

ARG ARCH

RUN mkdir -p /fs/usr/local/libexec/docker/cli-plugins
RUN case "${ARCH}" in \
    "amd64") \
    wget https://download.docker.com/linux/static/stable/x86_64/docker-28.1.1.tgz; \
    wget -O /fs/usr/local/libexec/docker/cli-plugins/docker-compose https://github.com/docker/compose/releases/download/v2.35.1/docker-compose-linux-x86_64; \
    ;; \
    "arm64") \
    wget https://download.docker.com/linux/static/stable/aarch64/docker-28.1.1.tgz; \
    wget -O /fs/usr/local/libexec/docker/cli-plugins/docker-compose https://github.com/docker/compose/releases/download/v2.35.1/docker-compose-linux-aarch64; \
    ;; \
    *) \
    exit 1 \
    ;; \
    esac

RUN chmod a+x /fs/usr/local/libexec/docker/cli-plugins/docker-compose

RUN mkdir -p /fs/usr/local/bin && tar -C /fs/usr/local/bin --strip-components=1 -xf docker-*.tgz docker/docker

FROM debian:bookworm-slim AS flecs

RUN apt-get update && \
    apt-get --yes --no-install-recommends install \
    ca-certificates nginx && \
    apt-get --yes clean && \
    rm -rf /var/lib/apt/lists/*

COPY --from=docker-downloader /fs /
ADD fs/ /

RUN echo "/usr/local/lib" >/etc/ld.so.conf.d/00-flecs.conf && ldconfig

FROM anchore/syft AS syft

COPY --from=flecs /usr/local/bin/flecsd /flecs/
WORKDIR /sbom

RUN ["/syft", "dir:/flecs", "-o", "spdx-json=./sbom.spdx.json"]

FROM flecs

COPY --from=syft /sbom /usr/local/lib/flecs/sbom

ENTRYPOINT [ "/entrypoint.sh" ]
