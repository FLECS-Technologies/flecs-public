FROM debian:bullseye-slim AS docker-downloader

ARG ARCH

ENV BASE_URL=dl.flecs.tech

COPY utils/docker/scripts/debian/install-packages.sh /tmp/

RUN /tmp/install-packages.sh ca-certificates wget

RUN [ "${ARCH}" = "amd64" ] && wget https://download.docker.com/linux/static/stable/x86_64/docker-24.0.4.tgz || true
RUN [ "${ARCH}" = "armhf" ] && wget https://download.docker.com/linux/static/stable/armhf/docker-24.0.4.tgz || true
RUN [ "${ARCH}" = "arm64" ] && wget https://download.docker.com/linux/static/stable/aarch64/docker-24.0.4.tgz || true

RUN mkdir -p /fs/usr/local/bin && tar -C /fs/usr/local/bin --strip-components=1 -xf docker-*.tgz docker/docker

FROM debian:bullseye-slim

COPY --from=docker-downloader /fs /
ADD fs/ /
ADD flecs-slim/fs/ /

ENTRYPOINT [ "/entrypoint.sh" ]
