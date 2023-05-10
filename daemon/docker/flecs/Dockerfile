FROM debian:bullseye-slim

ADD fs/ /
ADD flecs/fs/ /

COPY utils/docker/scripts/debian/install-packages.sh /tmp/

RUN /tmp/install-packages.sh ca-certificates

RUN /tmp/install-packages.sh containerd.io docker-ce-cli git \
    iptables libusb-1.0-0 procps

RUN \
    update-alternatives --set iptables /usr/sbin/iptables-legacy && \
    update-alternatives --set ip6tables /usr/sbin/ip6tables-legacy

ENTRYPOINT [ "/entrypoint.sh" ]
