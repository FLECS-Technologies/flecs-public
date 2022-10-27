FROM debian:bullseye-slim

ARG ARCH

COPY build-utils/docker/scripts/debian/install-packages.sh /tmp/

RUN /tmp/install-packages.sh ca-certificates libusb-1.0-0

ADD fs/ /

ENTRYPOINT [ "/entrypoint.sh" ]
