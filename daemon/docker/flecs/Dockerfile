FROM debian:bullseye-slim AS docker-downloader

ARG ARCH

ENV BASE_URL=https://s3-eu-central-2.ionoscloud.com/flecs-public/flecs-docker-ce

COPY utils/docker/scripts/debian/install-packages.sh /tmp/

RUN /tmp/install-packages.sh ca-certificates curl wget

RUN wget ${BASE_URL}/flecs-docker-ce_`curl -s -f ${BASE_URL}/latest-docker-ce`~debian.10~buster_${ARCH}.deb

RUN dpkg-deb -x *.deb /fs

FROM debian:bullseye-slim

ADD fs/ /
ADD flecs/fs/ /

COPY --from=docker-downloader /fs /

COPY utils/docker/scripts/debian/install-packages.sh /tmp/

RUN /tmp/install-packages.sh ca-certificates

RUN /tmp/install-packages.sh containerd.io docker-ce-cli git \
    iptables libusb-1.0-0 procps

RUN \
    update-alternatives --set iptables /usr/sbin/iptables-legacy && \
    update-alternatives --set ip6tables /usr/sbin/ip6tables-legacy

ENTRYPOINT [ "/entrypoint.sh" ]
