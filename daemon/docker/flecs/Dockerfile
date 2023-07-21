FROM debian:bullseye-slim AS docker-downloader

ARG ARCH

ENV BASE_URL=dl.flecs.tech

COPY utils/docker/scripts/debian/install-packages.sh /tmp/

RUN /tmp/install-packages.sh ca-certificates curl wget

RUN wget ${BASE_URL}/flecs-docker-ce/deb/flecs-docker-ce_`curl -fsSL ${BASE_URL}/latest-docker-ce`~debian.10~buster_${ARCH}.deb

RUN dpkg-deb -x *.deb /fs

FROM debian:bullseye-slim

ADD fs/ /
ADD flecs/fs/ /

COPY --from=docker-downloader /fs /

COPY utils/docker/scripts/debian/install-packages.sh /tmp/

RUN /tmp/install-packages.sh ca-certificates

RUN echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg]\
    https://download.docker.com/linux/debian buster stable" >\
    /etc/apt/sources.list.d/docker.list

RUN /tmp/install-packages.sh containerd.io docker-ce-cli git \
    iptables libusb-1.0-0 procps

RUN \
    update-alternatives --set iptables /usr/sbin/iptables-legacy && \
    update-alternatives --set ip6tables /usr/sbin/ip6tables-legacy

ENTRYPOINT [ "/entrypoint.sh" ]
